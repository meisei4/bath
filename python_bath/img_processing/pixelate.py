import os
from PIL import Image

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
INPUT_FILE = "../../Assets/Lights/2d_lights_and_shadows_neutral_point_light.webp"  # Input image file
OUTPUT_IMAGE_DIR = os.path.join(SCRIPT_DIR, "output/")
OUTPUT_FILE = os.path.join(OUTPUT_IMAGE_DIR, "output_image.png")
GRID_SIZE = 32  # Grid size for pixelation (e.g., 4x4 pixels will become 1 pixel)

def pixelate_image(input_file, output_file, grid_size):
    image = Image.open(input_file)
    width, height = image.size
    new_width = width // grid_size
    new_height = height // grid_size
    reduced_image = image.resize((new_width, new_height), Image.Resampling.NEAREST)
    pixelated_image = reduced_image.resize((width, height), Image.Resampling.NEAREST)
    pixelated_image.save(output_file)
    print(f"Pixelated image saved to {output_file}")

if __name__ == "__main__":
    pixelate_image(INPUT_FILE, OUTPUT_FILE, GRID_SIZE)
