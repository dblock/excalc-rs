# Unit Conversion

Simple linear (affine) conversions for temperature, distance, and mass.
Degree/radian conversion is already covered by `deg`/`rad` (see
[Standard Math](standard-math.md)) and isn't duplicated here.

| Function | Meaning | Example |
|----------|---------|---------|
| `c2f(c)` | Celsius to Fahrenheit | `c2f(100)` → `212` |
| `f2c(f)` | Fahrenheit to Celsius | `f2c(212)` → `100` |
| `km2mi(km)` | Kilometers to miles | `km2mi(1)` → `~0.6214` |
| `mi2km(mi)` | Miles to kilometers | `mi2km(1)` → `1.609344` |
| `kg2lb(kg)` | Kilograms to pounds | `kg2lb(1)` → `~2.2046` |
| `lb2kg(lb)` | Pounds to kilograms | `lb2kg(1)` → `0.45359237` |
| `m2ft(m)` | Meters to feet | `m2ft(1)` → `~3.2808` |
| `ft2m(ft)` | Feet to meters | `ft2m(1)` → `0.3048` |

## Notes and implementation details

- `km2mi`/`mi2km` use the exact conversion factor `1 mi = 1.609344 km`.
- `kg2lb`/`lb2kg` use the exact conversion factor `1 lb = 0.45359237 kg`.
- `m2ft`/`ft2m` use the exact conversion factor `1 ft = 0.3048 m`.
- All conversions are simple multiplication/division (or, for temperature, an affine transform) — no rounding or unit-system inference.
