use super::{
    ALLOY_GRADIENT_END, ALLOY_GRADIENT_MID, ALLOY_GRADIENT_START, SIMULATED_BLUR_RADIUS,
};

fn extract_rgb(color: u32) -> (u8, u8, u8) {
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;
    (r, g, b)
}

fn make_argb(a: u8, r: u8, g: u8, b: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

fn blend_colors(base: u32, overlay: u32, alpha: u8) -> u32 {
    let (br, bg, bb) = extract_rgb(base);
    let (or, og, ob) = extract_rgb(overlay);
    let a = alpha as u32;
    let inv_a = 255 - a;
    let r = ((br as u32 * inv_a + or as u32 * a) / 255) as u8;
    let g = ((bg as u32 * inv_a + og as u32 * a) / 255) as u8;
    let b = ((bb as u32 * inv_a + ob as u32 * a) / 255) as u8;
    make_argb(0xFF, r, g, b)
}

pub fn build_gradient_background(width: u32, height: u32) -> alloc::vec::Vec<u32> {
    let mut pixels = alloc::vec![ALLOY_GRADIENT_START; (width * height) as usize];

    let (sr, sg, sb) = extract_rgb(ALLOY_GRADIENT_START);
    let (mr, mg, mb) = extract_rgb(ALLOY_GRADIENT_MID);
    let (er, eg, eb) = extract_rgb(ALLOY_GRADIENT_END);

    let mid_y = height * 40 / 100;

    for y in 0..height {
        let (r, g, b) = if y <= mid_y {
            if mid_y == 0 {
                (mr, mg, mb)
            } else {
                let t = (y * 255 / mid_y) as u32;
                (
                    (sr as u32 + (mr as u32).saturating_sub(sr as u32) * t / 255) as u8,
                    (sg as u32 + (mg as u32).saturating_sub(sg as u32) * t / 255) as u8,
                    (sb as u32 + (mb as u32).saturating_sub(sb as u32) * t / 255) as u8,
                )
            }
        } else {
            let remaining = height.saturating_sub(mid_y);
            if remaining == 0 {
                (er, eg, eb)
            } else {
                let t = ((y - mid_y) * 255 / remaining) as u32;
                (
                    (mr as u32 + (er as u32).saturating_sub(mr as u32) * t / 255) as u8,
                    (mg as u32 + (eg as u32).saturating_sub(mg as u32) * t / 255) as u8,
                    (mb as u32 + (eb as u32).saturating_sub(mb as u32) * t / 255) as u8,
                )
            }
        };

        let row_color = make_argb(0xFF, r, g, b);
        let row_start = (y * width) as usize;
        let row_end = row_start + width as usize;
        for px in pixels[row_start..row_end].iter_mut() {
            *px = row_color;
        }
    }

    pixels
}

fn integer_sqrt(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = 1;
    while y < x {
        x = y;
        y = (n / y + y) / 2;
    }
    x
}

pub fn draw_rounded_rect(
    pixels: &mut [u32],
    stride: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    radius: u32,
    color: u32,
) {
    if stride == 0 || width == 0 || height == 0 {
        return;
    }

    let radius = radius.min(width / 2).min(height / 2);
    let max_y = (pixels.len() as u32).saturating_div(stride);

    for dy in 0..height {
        let py = y + dy;
        if py >= max_y {
            break;
        }

        let mut start_x = x;
        let mut end_x = x + width;

        if dy < radius {
            let dist = radius - dy;
            let corner_offset = integer_sqrt(dist);
            start_x = x + corner_offset;
            end_x = x + width - corner_offset;
        } else if dy >= height - radius {
            let dist = radius - (height - 1 - dy);
            let corner_offset = integer_sqrt(dist);
            start_x = x + corner_offset;
            end_x = x + width - corner_offset;
        }

        start_x = start_x.max(x);
        end_x = end_x.min(x + width);

        let row_offset = (py * stride) as usize;
        for px in start_x..end_x {
            if px < stride {
                pixels[row_offset + px as usize] = color;
            }
        }
    }
}

pub fn fill_rounded_rect(
    pixels: &mut [u32],
    stride: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    radius: u32,
    color: u32,
) {
    draw_rounded_rect(pixels, stride, x, y, width, height, radius, color);
}

pub fn apply_simulated_blur(
    pixels: &mut [u32],
    stride: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    tint_color: u32,
    tint_alpha: u8,
) {
    if stride == 0 || width == 0 || height == 0 {
        return;
    }

    let max_y = (pixels.len() as u32).saturating_div(stride);
    let step = SIMULATED_BLUR_RADIUS;

    let region_y_start = y.min(max_y);
    let region_y_end = (y + height).min(max_y);
    let region_x_end = (x + width).min(stride);

    let mut sample_r: u32 = 0;
    let mut sample_g: u32 = 0;
    let mut sample_b: u32 = 0;
    let mut sample_count: u32 = 0;

    for dy in (0..height).step_by(step as usize) {
        let py = region_y_start + dy;
        if py >= region_y_end {
            break;
        }
        for dx in (0..width).step_by(step as usize) {
            let px = x + dx;
            if px >= region_x_end {
                break;
            }
            let offset = (py * stride + px) as usize;
            let color = pixels[offset];
            let (r, g, b) = extract_rgb(color);
            sample_r += r as u32;
            sample_g += g as u32;
            sample_b += b as u32;
            sample_count += 1;
        }
    }

    if sample_count == 0 {
        return;
    }

    let avg_r = (sample_r / sample_count) as u8;
    let avg_g = (sample_g / sample_count) as u8;
    let avg_b = (sample_b / sample_count) as u8;
    let avg_color = make_argb(0xFF, avg_r, avg_g, avg_b);

    for dy in 0..height {
        let py = region_y_start + dy;
        if py >= region_y_end {
            break;
        }
        let row_offset = (py * stride) as usize;
        for dx in 0..width {
            let px = x + dx;
            if px >= region_x_end {
                break;
            }
            pixels[row_offset + px as usize] = blend_colors(pixels[row_offset + px as usize], avg_color, tint_alpha);
        }
    }

    let (tr, tg, tb) = extract_rgb(tint_color);
    let tint = make_argb(0xFF, tr, tg, tb);
    for dy in 0..height {
        let py = region_y_start + dy;
        if py >= region_y_end {
            break;
        }
        let row_offset = (py * stride) as usize;
        for dx in 0..width {
            let px = x + dx;
            if px >= region_x_end {
                break;
            }
            pixels[row_offset + px as usize] = blend_colors(pixels[row_offset + px as usize], tint, tint_alpha / 2);
        }
    }
}

pub fn draw_shadow_edge(
    pixels: &mut [u32],
    stride: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) {
    if stride == 0 || width == 0 || height == 0 {
        return;
    }

    let max_y = (pixels.len() as u32).saturating_div(stride);
    let shadow_color = 0x40000000;

    let bottom_y = y + height;
    if bottom_y < max_y {
        let row_offset = (bottom_y * stride) as usize;
        for dx in 0..width {
            let px = x + dx;
            if px < stride {
                pixels[row_offset + px as usize] = shadow_color;
            }
        }
    }

    let right_x = x + width;
    if right_x < stride {
        for dy in 0..height {
            let py = y + dy;
            if py < max_y {
                let offset = (py * stride + right_x) as usize;
                pixels[offset] = shadow_color;
            }
        }
    }
}
