# Standard Math

The circular trig functions below operate in **radians only**. Rather than a global degree/radian mode toggle (as the original Pascal engine had — see [../../port/DESIGN.md](../../port/DESIGN.md#angle-units)), degree support is explicit and stateless: use [`deg`/`rad`](#degreeradian-conversion) to convert, or the `sind`/`cosd`/`tand`/`asind`/`acosd`/`atand` degree-native variants below.

## Trigonometric

| Function | Meaning | Domain | Example |
|----------|---------|--------|---------|
| `sin(x)` | Sine | all reals | `sin(pi/2)` → `1` |
| `cos(x)` | Cosine | all reals | `cos(0)` → `1` |
| `tan(x)` | Tangent | `cos(x) != 0` | `tan(pi/4)` → `~1` |
| `asin(x)` | Arcsine | `-1 <= x <= 1` | `asin(1)` → `pi/2` |
| `acos(x)` | Arccosine | `-1 <= x <= 1` | `acos(0)` → `pi/2` |
| `atan(x)` | Arctangent | all reals | `atan(1)` → `pi/4` |

## Degree/radian conversion

| Function | Meaning | Domain | Example |
|----------|---------|--------|---------|
| `deg(x)` | Converts radians to degrees | all reals | `deg(pi)` → `180` |
| `rad(x)` | Converts degrees to radians | all reals | `rad(180)` → `pi` |
| `sind(x)` | Sine, `x` in degrees | all reals | `sind(30)` → `0.5` |
| `cosd(x)` | Cosine, `x` in degrees | all reals | `cosd(60)` → `0.5` |
| `tand(x)` | Tangent, `x` in degrees | `cosd(x) != 0` | `tand(45)` → `~1` |
| `asind(x)` | Arcsine, result in degrees | `-1 <= x <= 1` | `asind(0.5)` → `30` |
| `acosd(x)` | Arccosine, result in degrees | `-1 <= x <= 1` | `acosd(0.5)` → `60` |
| `atand(x)` | Arctangent, result in degrees | all reals | `atand(1)` → `45` |

## Reciprocal trigonometric

| Function | Meaning | Domain | Example |
|----------|---------|--------|---------|
| `sec(x)` | Secant (`1/cos`) | `cos(x) != 0` | `sec(0)` → `1` |
| `csc(x)` | Cosecant (`1/sin`) | `sin(x) != 0` | `csc(pi/2)` → `1` |
| `cot(x)` | Cotangent (`cos/sin`) | `sin(x) != 0` | `cot(pi/4)` → `~1` |
| `asec(x)` | Inverse secant | `x != 0`, `1/x` in `[-1, 1]` | `asec(1)` → `0` |
| `acsc(x)` | Inverse cosecant | `x != 0`, `1/x` in `[-1, 1]` | `acsc(1)` → `pi/2` |
| `acot(x)` | Inverse cotangent | `x != 0` | `acot(1)` → `pi/4` |

## Hyperbolic

| Function | Meaning | Domain | Example |
|----------|---------|--------|---------|
| `sinh(x)` | Hyperbolic sine | all reals | `sinh(0)` → `0` |
| `cosh(x)` | Hyperbolic cosine | all reals | `cosh(0)` → `1` |
| `tanh(x)` | Hyperbolic tangent | all reals | `tanh(0)` → `0` |
| `asinh(x)` | Inverse hyperbolic sine | all reals | `asinh(0)` → `0` |
| `acosh(x)` | Inverse hyperbolic cosine | `x >= 1` | `acosh(1)` → `0` |
| `atanh(x)` | Inverse hyperbolic tangent | `-1 <= x <= 1` | `atanh(0)` → `0` |

## Reciprocal hyperbolic

| Function | Meaning | Domain | Example |
|----------|---------|--------|---------|
| `sech(x)` | Hyperbolic secant | `cosh(x) != 0` (always true) | `sech(0)` → `1` |
| `csch(x)` | Hyperbolic cosecant | `x != 0` | `csch(1)` → `~0.851` |
| `coth(x)` | Hyperbolic cotangent | `x != 0` | `coth(1)` → `~1.313` |
| `asech(x)` | Inverse hyperbolic secant | `0 < x <= 1` | `asech(1)` → `0` |
| `acsch(x)` | Inverse hyperbolic cosecant | `x != 0` | `acsch(1)` → `~0.881` |
| `acoth(x)` | Inverse hyperbolic cotangent | `\|x\| > 1` | `acoth(2)` → `~0.549` |

## Logarithmic and root

| Function | Meaning | Domain | Example |
|----------|---------|--------|---------|
| `sqrt(x)` | Square root | `x >= 0` | `sqrt(16)` → `4` |
| `ln(x)` | Natural logarithm (base e) | `x > 0` | `ln(e)` → `1` |
| `log(x)` | Common logarithm (base 10) | `x > 0` | `log(100)` → `2` |
| `logn(x, b)` | Logarithm of `x` in base `b` | `x > 0`, `b > 0`, `b != 1` | `logn(8, 2)` → `3` |

Note: in the original Pascal source, `MyLog` computes base-10 log (not natural log) — `log` in this port preserves that semantics deliberately, despite the name looking like it could mean `ln`. Use `ln` for natural log.
