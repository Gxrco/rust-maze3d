use crate::framebuffer::Framebuffer;
use crate::player::Player;

pub struct Intersect {
    pub distance: f32,
    pub impact: char,
    pub tx: usize,
}

pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &[Vec<char>],
    player: &Player,
    a: f32,
    block_size: usize,
    draw_line: bool,
) -> Intersect {
    let mut d = 0.0;

    framebuffer.set_current_color(0xFFFFFF);

    let cos_a = a.cos();
    let sin_a = a.sin();

    loop {
        let cos_d = d * cos_a;
        let sin_d = d * sin_a;

        let x = (player.pos.x + cos_d) as usize;
        let y = (player.pos.y + sin_d) as usize;

        let i = x / block_size;
        let j = y / block_size;

        let hitx = x % block_size;
        let hity = y % block_size;

        let maxhit = if hitx > 1 && hitx < block_size - 1 {
            hitx
        } else {
            hity
        };

        if draw_line {
            framebuffer.point(x, y);
        }

        if let Some(&impact) = maze.get(j).and_then(|row| row.get(i)) {
            if impact != ' ' {
                return Intersect {
                    distance: d,
                    impact,
                    tx: maxhit * 128 / block_size,
                };
            }
        }

        d += 2.0;
    }
}
