from PIL import Image
import struct
import pathlib

def convert(input: str, output: str):
    img = Image.open(input)

    buffer = bytearray(img.height*img.width*2)
    for y in range(img.height):
        for x in range(img.width):
            pixel = img.getpixel((x, y))
            rgb565 = ((pixel[0] >> 3) << 11) | ((pixel[1] >> 2) << 5) | ((pixel[2] >> 3) << 0)
            struct.pack_into("<H", buffer, (x + y * img.width) * 2, rgb565)

    with open(output, "wb") as f:
        f.write(buffer)

import glob
for input in glob.glob("*.bmp"):
    input_path = pathlib.Path(input)
    output_path = input_path.with_suffix(".raw")
    convert(input, str(output_path))