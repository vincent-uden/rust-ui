import click
import os
import glob
from PIL import Image
import csv

@click.command()
@click.argument('directory', type=click.Path(exists=True, file_okay=False))
@click.option('--grid-cols', default=10, type=int, help='Number of columns in the grid')
@click.option('--tile-size', default=40, type=int, help='Size of each tile in pixels (square)')
@click.option('--output', default='atlas', help='Base name for output files (atlas.png and atlas.csv)')
def main(directory, grid_cols, tile_size, output):
    png_files = glob.glob(os.path.join(directory, '*.png'))
    if not png_files:
        click.echo("No PNG files found in the directory.")
        return

    images = []
    for f in png_files:
        img = Image.open(f)
        name = os.path.splitext(os.path.basename(f))[0]
        images.append((name, img))

    num_images = len(images)
    rows = (num_images + grid_cols - 1) // grid_cols
    atlas_width = grid_cols * tile_size
    atlas_height = rows * tile_size

    atlas = Image.new('RGBA', (atlas_width, atlas_height), (0, 0, 0, 0))

    csv_data = []
    for i, (name, img) in enumerate(images):
        row = i // grid_cols
        col = i % grid_cols
        x = col * tile_size
        y = row * tile_size

        # Center the image in the tile
        offset_x = (tile_size - img.width) // 2
        offset_y = (tile_size - img.height) // 2
        paste_x = x + offset_x
        paste_y = y + offset_y

        atlas.paste(img, (paste_x, paste_y))
        csv_data.append([name, paste_x, paste_y, img.width, img.height])

    atlas_output = f"{output}.png"
    csv_output = f"{output}.csv"

    atlas.save(atlas_output)
    with open(csv_output, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(['name', 'x', 'y', 'width', 'height'])
        writer.writerows(csv_data)

    click.echo(f"Atlas saved to {atlas_output}, CSV saved to {csv_output}")

if __name__ == "__main__":
    main()
