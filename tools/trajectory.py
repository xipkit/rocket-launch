#!/usr/bin/env python3
"""Back-of-the-envelope orbit numbers for a Kestrel flight.

Given a target apogee and perigee (km above the surface) this prints the
orbital period, the velocities at both apsides, and the delta-v for a
Hohmann circularisation at apogee.
"""

import argparse
import math

MU_EARTH = 3.986004418e14  # m^3/s^2
R_EARTH = 6_371_000.0  # m


def vis_viva(r: float, a: float) -> float:
    """Speed at radius r on an orbit with semi-major axis a."""
    return math.sqrt(MU_EARTH * (2.0 / r - 1.0 / a))


def period(a: float) -> float:
    return 2.0 * math.pi * math.sqrt(a**3 / MU_EARTH)


def circularise_at_apogee(apogee_km: float, perigee_km: float) -> dict[str, float]:
    ra = R_EARTH + apogee_km * 1000.0
    rp = R_EARTH + perigee_km * 1000.0
    a = (ra + rp) / 2.0
    v_apogee = vis_viva(ra, a)
    v_circular = math.sqrt(MU_EARTH / ra)
    return {
        "period_min": period(a) / 60.0,
        "v_perigee_ms": vis_viva(rp, a),
        "v_apogee_ms": v_apogee,
        "dv_circularise_ms": v_circular - v_apogee,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apogee", type=float, required=True, help="km")
    parser.add_argument("--perigee", type=float, required=True, help="km")
    args = parser.parse_args()
    if args.perigee > args.apogee:
        parser.error("perigee cannot exceed apogee")

    numbers = circularise_at_apogee(args.apogee, args.perigee)
    for key, value in numbers.items():
        print(f"{key:>20}: {value:10.2f}")


if __name__ == "__main__":
    main()
