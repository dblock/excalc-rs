# Geometry

`distance`, `manhattan`, and `dot` work in any number of dimensions: pass a
single, even-length argument list split in half, the first half giving the
coordinates of point/vector `a` and the second half point/vector `b` (e.g.
`distance(1, 2, 4, 6)` is the 2D distance between `(1, 2)` and `(4, 6)`;
`distance(0, 0, 0, 1, 1, 1)` is the 3D distance between the origin and
`(1, 1, 1)`). `norm` takes a single n-dimensional vector directly.

| Function | Meaning | Domain | Example |
|----------|---------|--------|---------|
| `distance(a..., b...)` | Euclidean distance between two n-dimensional points | `R^2n -> R+` (`n >= 1`) | `distance(0, 0, 3, 4)` → `5` |
| `manhattan(a..., b...)` | Manhattan (taxicab, L1) distance between two n-dimensional points | `R^2n -> R+` (`n >= 1`) | `manhattan(0, 0, 3, 4)` → `7` |
| `dot(a..., b...)` | Dot product of two n-dimensional vectors | `R^2n -> R` (`n >= 1`) | `dot(1, 2, 3, 4)` → `11` |
| `norm(v...)` | Euclidean norm (magnitude) of an n-dimensional vector | `R^n -> R+` (`n >= 1`) | `norm(3, 4)` → `5` |
| `triarea(a, b, c)` | Area of a triangle with side lengths `a`, `b`, `c` (Heron's formula) | `R+^3 -> R+` | `triarea(3, 4, 5)` → `6` |
| `circlearea(r)` | Area of a circle of radius `r` | `R+ -> R+` | `circlearea(2)` → `12.566...` |
| `circumference(r)` | Circumference of a circle of radius `r` | `R+ -> R+` | `circumference(2)` → `12.566...` |
| `spherevol(r)` | Volume of a sphere of radius `r` | `R+ -> R+` | `spherevol(3)` → `113.097...` |
| `spherearea(r)` | Surface area of a sphere of radius `r` | `R+ -> R+` | `spherearea(3)` → `113.097...` |

## Notes and implementation details

- `distance`/`manhattan`/`dot` require an even number of arguments (at least `2`), split evenly in half; an odd count or fewer than `2` arguments is a `WrongArgCount` error. There's no dedicated 2D/3D-only variant — the same function handles any dimension.
- `triarea` validates the triangle inequality (`a + b > c` and permutations) and rejects non-positive side lengths before applying Heron's formula.
- `circlearea`/`circumference`/`spherevol`/`spherearea` all require `r > 0`.
- A dedicated 3D vector `cross` product was considered but omitted: it returns a vector, not a scalar, and the calculator has no vector-valued result type (only numbers and, for base conversion, formatted text).
