//! Turning a thumb position into a ring and a direction, with hysteresis.

use crate::layout::RingId;

/// Radius of the dead zone, as a fraction of the wheel radius. Releasing inside it
/// types the centre key.
pub const DEAD_ZONE: f32 = 0.15;
/// Border between the inner and outer ring, as a fraction of the wheel radius.
pub const INNER_EDGE: f32 = 0.50;
/// Default margin past a ring border before the selection switches (fraction of R).
pub const DEFAULT_RING_MARGIN: f32 = 0.03;
/// Extra degrees a direction keeps before switching to the next one.
pub const DIRECTION_HOLD_DEG: f32 = 3.0;
/// Width of the band next to a sector border where prediction may correct (degrees).
pub const BORDER_BAND_DEG: f32 = 8.0;
/// Width of the bell used to weigh the angle against prediction (degrees).
pub const SIGMA_DEG: f32 = 12.0;

/// Angle in degrees, 0 at the top, clockwise, in `[0, 360)`.
pub fn angle_of(dx: f32, dy: f32) -> f32 {
    (dy.atan2(dx).to_degrees() + 90.0).rem_euclid(360.0)
}

/// Signed difference `a − b` in degrees, in `[-180, 180)`.
pub fn angle_diff(a: f32, b: f32) -> f32 {
    (a - b + 540.0).rem_euclid(360.0) - 180.0
}

pub(crate) fn gauss(d: f32) -> f32 {
    (-(d * d) / (2.0 * SIGMA_DEG * SIGMA_DEG)).exp()
}

/// Raw hit: where the thumb is, before any correction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub ring: RingId,
    pub dir: u8,
    /// Thumb angle in degrees (0 = up, clockwise).
    pub angle: f32,
}

/// Where the thumb is relative to where it touched down (`dx`, `dy` in screen
/// pixels, y pointing down) on a wheel of `radius` pixels. `prev` is the last hit,
/// so a selection only changes once the thumb is clearly past a border.
/// Returns `None` inside the dead zone.
pub fn hit(prev: Option<Hit>, dx: f32, dy: f32, radius: f32, margin: f32) -> Option<Hit> {
    let dist = dx.hypot(dy) / radius.max(1.0);
    let angle = angle_of(dx, dy);
    let dead = DEAD_ZONE - if prev.is_some() { margin / 2.0 } else { 0.0 };
    if dist < dead {
        return None;
    }
    let mut ring = if dist < INNER_EDGE {
        RingId::Inner
    } else {
        RingId::Outer
    };
    if let Some(p) = prev {
        match p.ring {
            RingId::Inner if dist < INNER_EDGE + margin => ring = RingId::Inner,
            RingId::Outer if dist >= INNER_EDGE - margin => ring = RingId::Outer,
            _ => {}
        }
    }
    let mut dir = ((angle / 45.0).round() as u8) % 8;
    if let Some(p) = prev {
        if p.ring == ring
            && angle_diff(angle, p.dir as f32 * 45.0).abs() < 22.5 + DIRECTION_HOLD_DEG
        {
            dir = p.dir;
        }
    }
    Some(Hit { ring, dir, angle })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(angle: f32, dist: f32) -> (f32, f32) {
        let r = angle.to_radians();
        (dist * r.sin() * 100.0, -dist * r.cos() * 100.0)
    }

    #[test]
    fn up_is_zero_and_clockwise() {
        assert!((angle_of(0.0, -1.0) - 0.0).abs() < 1e-3);
        assert!((angle_of(1.0, 0.0) - 90.0).abs() < 1e-3);
        assert!((angle_of(0.0, 1.0) - 180.0).abs() < 1e-3);
    }

    #[test]
    fn rings_have_hysteresis() {
        let (x, y) = at(0.0, 0.52);
        let h = hit(None, x, y, 100.0, 0.03).unwrap();
        assert_eq!(h.ring, RingId::Outer);
        // Coming back from the outer ring, 0.49 R still counts as outer…
        let (x, y) = at(0.0, 0.49);
        assert_eq!(hit(Some(h), x, y, 100.0, 0.03).unwrap().ring, RingId::Outer);
        // …and 0.45 R is inner again.
        let (x, y) = at(0.0, 0.45);
        assert_eq!(hit(Some(h), x, y, 100.0, 0.03).unwrap().ring, RingId::Inner);
    }

    #[test]
    fn directions_hold_a_few_degrees() {
        let (x, y) = at(10.0, 0.3);
        let h = hit(None, x, y, 100.0, 0.03).unwrap();
        assert_eq!(h.dir, 0);
        let (x, y) = at(24.0, 0.3);
        assert_eq!(hit(Some(h), x, y, 100.0, 0.03).unwrap().dir, 0);
        let (x, y) = at(27.0, 0.3);
        assert_eq!(hit(Some(h), x, y, 100.0, 0.03).unwrap().dir, 1);
    }

    #[test]
    fn dead_zone_is_none() {
        assert!(hit(None, 5.0, 5.0, 100.0, 0.03).is_none());
    }
}
