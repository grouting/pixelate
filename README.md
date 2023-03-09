# Usage

The basic structure of a pixelate command is as follows:

```bash
pixelate <input> <scale_factor> [options]
```

 | Option | Description |
 | --- | --- |
 | `-k` | Keep the dimensions of the output image the same as the input |
 | `-f` | Force crop the image in order for it to be divisible by the scale factor |
 | `-c` | Centre the image if cropping is required |
 | `-a` | Use all optional flags |

## Examples

```bash
pixelate sky.png 5 -k -f
```

Retains the dimensions, and crops the image to be divisible by the scale factor

```bash
pixelate burning_pits_of_hell.png 5 -a
```

Centres the image, retains the dimensions, and crops the image to be divisible by the scale factor
