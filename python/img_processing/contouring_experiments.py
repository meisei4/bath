import cv2
import numpy as np
from PIL import Image
import matplotlib.pyplot as plt
from matplotlib.gridspec import GridSpec
from enum import Enum


class ColorTheme(Enum):
    # Star Trek Lcars attempt
    BLACK = 'black'
    ORANGE = 'orange'
    RED = 'red'
    CYAN = 'cyan'


CURRENT_THEME = ColorTheme

CONFIG = {
    "OFFSET_DIST": 0.5,
    "ANGLE_SCALE": 2.0,
    "DISPLAY": True,
    "COLOR_THEME": CURRENT_THEME
}

IMAGE_FILE = "../../Assets/Sprites/tile_0006.png"


def offset_polygon_vertices(verts):
    if len(verts) < 3:
        return verts.copy()

    out = []
    n = len(verts)
    for i in range(n):
        p0 = verts[(i - 1) % n]
        p1 = verts[i]
        p2 = verts[(i + 1) % n]

        e0 = p1 - p0
        e1 = p2 - p1
        l0, l1 = np.hypot(*e0), np.hypot(*e1)
        if l0 < 1e-9 or l1 < 1e-9:
            out.append(p1.copy())
            continue

        d0, d1 = e0 / l0, e1 / l1
        n0 = np.array([-d0[1], d0[0]])
        n1 = np.array([-d1[1], d1[0]])
        bisector = n0 + n1
        mag = np.hypot(*bisector)

        offset_pt = p1 + (
            CONFIG["OFFSET_DIST"] * (bisector / mag) * CONFIG["ANGLE_SCALE"]
            if mag >= 1e-9 else CONFIG["OFFSET_DIST"] * n0
        )
        out.append(offset_pt)
    return np.array(out)


def process_contour_vertex_outward(path):
    img = Image.open(path).convert("RGBA")
    width, height = img.size
    alpha = np.array(img.split()[-1])
    mask = (alpha > 0).astype(np.uint8) * 255

    contours, _ = cv2.findContours(mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    if not contours:
        raise ValueError("No contour found.")

    raw = max(contours, key=cv2.contourArea).reshape(-1, 2).astype(float)

    area = sum(raw[i, 0] * raw[(i + 1) % len(raw), 1] - raw[i, 1] * raw[(i + 1) % len(raw), 0] for i in range(len(raw)))
    if area > 0:
        raw = raw[::-1]

    raw += 0.5
    outward = offset_polygon_vertices(raw)
    uv_raw = raw / [width, height]
    uv_out = outward / [width, height]

    print("var uvs_outward_shift: PackedVector2Array = [")
    for uv in uv_out:
        print(f"    Vector2({uv[0]:.4f}, {uv[1]:.4f}),")
    print("]\n")

    if CONFIG["DISPLAY"]:
        theme = CONFIG["COLOR_THEME"]
        fig = plt.figure(figsize=(18, 6))
        fig.patch.set_facecolor(theme.BLACK.value)

        gs = GridSpec(1, 3, width_ratios=[1, 1, 1], figure=fig)
        gs.update(wspace=0.05, hspace=0.05)  # Minimal whitespace

        ax1 = fig.add_subplot(gs[0, 0])
        ax2 = fig.add_subplot(gs[0, 1])
        ax3 = fig.add_subplot(gs[0, 2])

        axes = [ax1, ax2, ax3]

        for ax in axes:
            ax.set_facecolor(theme.BLACK.value)
            ax.spines['bottom'].set_color(theme.ORANGE.value)
            ax.spines['left'].set_color(theme.ORANGE.value)
            ax.tick_params(axis='x', colors=theme.ORANGE.value)
            ax.tick_params(axis='y', colors=theme.ORANGE.value)
            ax.title.set_color(theme.ORANGE.value)
            ax.xaxis.label.set_color(theme.ORANGE.value)
            ax.yaxis.label.set_color(theme.ORANGE.value)

        # 1) Texture in MODEL SPACE
        ax1.imshow(img)
        ax1.scatter(raw[:, 0], raw[:, 1], marker='s', c=theme.RED.value, s=2, label="Raw")
        ax1.scatter(outward[:, 0], outward[:, 1], marker='s', c=theme.CYAN.value, s=6, label="Outward")
        ax1.set_title("MODEL SPACE")

        # 2) Normal Space (UVs) - Center Plot
        ax2.scatter(uv_raw[:, 0], uv_raw[:, 1], marker='s', c=theme.RED.value, s=2, label="UV Raw")
        ax2.scatter(uv_out[:, 0], uv_out[:, 1], marker='s', c=theme.CYAN.value, s=6, label="UV Outward")
        ax2.set_xlim(0, 1)
        ax2.set_ylim(1, 0)
        ax2.set_aspect('equal', 'box')  # Ensure the center plot is square
        ax2.set_title("NORMAL SPACE (UVs)")
        ax2.legend(facecolor=theme.BLACK.value, edgecolor=theme.ORANGE.value, loc='upper right')
        for text in ax2.get_legend().get_texts():
            text.set_color(theme.ORANGE.value)

        # 3) UVs in MODEL SPACE (same size as texture)
        ax3.scatter(raw[:, 0], raw[:, 1], marker='s', c=theme.RED.value, s=2, label="Raw")
        ax3.scatter(outward[:, 0], outward[:, 1], marker='s', c=theme.CYAN.value, s=6, label="Outward")
        ax3.set_xlim(0, width)
        ax3.set_ylim(height, 0)  # Flip vertically
        ax3.set_aspect('equal', 'box')
        ax3.set_title("MODEL SPACE (UVs)")

        plt.show()

    return uv_out


if __name__ == "__main__":
    process_contour_vertex_outward(IMAGE_FILE)
