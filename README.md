# Usage

The basic structure of a pixelate command is as follows:

```bash
pixelate <path> <scale_factor> [options]
```

 | Option | Description |
 | --- | --- |
 | `-k` | Keep the dimensions of the output image the same as the input |
 | `-f` | Force crop the image in order for it to be divisible by the scale factor |
 | `-c` | Centre the image if cropping is required |
 | `-o` | Overwrite the input image |
 | `-a` | Use all optional flags |

## Examples

```bash
pixelate sky.png 5 -k -f
```

Retains the dimensions, and crops the image to be divisible by the scale factor

```bash
pixelate burning_pits_of_hell.png 5 -a
```

Centres the image, retains the dimensions, crops the image in order for it to be divisible by the scale factor and overwrites the original image

> Equivalent to
>
> ```bash
> pixelate burning_pits_of_hell.png 5 -k -f -c -o
>```
