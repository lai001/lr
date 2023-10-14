pub fn catmull_rom(t: f32, alpha: f32, c0: f32, c1: f32, p0: f32, p1: f32) -> f32 {
    glam::vec4(t * t * t, t * t, t, 1.0).dot(
        glam::mat4(
            glam::vec4(-alpha, 2.0 * alpha, -alpha, 0.0),
            glam::vec4(2.0 - alpha, alpha - 3.0, 0.0, 1.0),
            glam::vec4(alpha - 2.0, 3.0 - 2.0 * alpha, alpha, 0.0),
            glam::vec4(alpha, -alpha, 0.0, 0.0),
        ) * glam::vec4(c0, p0, p1, c1),
    )
}

pub fn catmull_rom_vec2(
    t: f32,
    alpha: f32,
    c0: glam::Vec2,
    c1: glam::Vec2,
    p0: glam::Vec2,
    p1: glam::Vec2,
) -> glam::Vec2 {
    let x = catmull_rom(t, alpha, c0.x, c1.x, p0.x, p1.x);
    let y = catmull_rom(t, alpha, c0.y, c1.y, p0.y, p1.y);
    glam::vec2(x, y)
}

pub fn catmull_rom_vec2_array(
    t: f32,
    alpha: f32,
    c0: &glam::Vec2,
    c1: &glam::Vec2,
    points: &[glam::Vec2],
) -> glam::Vec2 {
    debug_assert!(points.len() >= 2);

    let mut group: Vec<&glam::Vec2> = Vec::new();
    group.push(c0);
    for point in points {
        group.push(point);
    }
    group.push(c1);

    for i in 0..group.len() - 3 {
        catmull_rom_vec2(t, alpha, *group[i], *group[i], *group[i], *group[i]);
    }

    todo!()
}

pub fn catmull_rom_vec3(
    t: f32,
    alpha: f32,
    c0: glam::Vec3,
    c1: glam::Vec3,
    p0: glam::Vec3,
    p1: glam::Vec3,
) -> glam::Vec3 {
    let x = catmull_rom(t, alpha, c0.x, c1.x, p0.x, p1.x);
    let y = catmull_rom(t, alpha, c0.y, c1.y, p0.y, p1.y);
    let z = catmull_rom(t, alpha, c0.z, c1.z, p0.z, p1.z);
    glam::vec3(x, y, z)
}

#[cfg(test)]
mod test {
    use super::*;
    use plotters::prelude::*;

    #[test]
    fn catmull_rom_vec2_test() {
        let c0 = glam::vec2(0.0, 1.0);
        let c1 = glam::vec2(1.0, 1.0);
        let p0 = glam::vec2(0.25, 0.5);
        let p1 = glam::vec2(0.75, 0.5);
        let alpha = 0.5;

        let results = (0..100)
            .map(|x| {
                let t = x as f32 / 100.0;
                catmull_rom_vec2(t, alpha, c0, c1, p0, p1)
            })
            .collect::<Vec<glam::Vec2>>();

        let filename = "target/catmull_rom_vec2_test.png";
        let root = IntoDrawingArea::into_drawing_area(BitMapBackend::new(filename, (512, 512)));
        root.fill(&plotters::style::WHITE).unwrap();
        let mut chart = ChartBuilder::on(&root)
            .margin(15)
            .x_label_area_size(25)
            .y_label_area_size(25)
            .build_cartesian_2d(0_f32..1.2_f32, 0_f32..1_f32)
            .unwrap();

        chart.configure_mesh().draw().unwrap();

        let _ = chart.draw_series(PointSeries::of_element(
            vec![c0, c1].iter().map(|v| (v.x as f32, v.y as f32)),
            5.0,
            &plotters::style::BLUE,
            &|c, s, st| {
                return EmptyElement::at(c)
                    + Circle::new((0, 0), s, st.filled())
                    + Text::new(format!("{:?}", c), (20, 0), ("sans-serif", 20).into_font());
            },
        ));

        let _ = chart.draw_series(PointSeries::of_element(
            vec![p0, p1].iter().map(|v| (v.x as f32, v.y as f32)),
            5.0,
            &plotters::style::RED,
            &|c, s, st| {
                return EmptyElement::at(c)
                    + Circle::new((0, 0), s, st.filled())
                    + Text::new(format!("{:?}", c), (20, 0), ("sans-serif", 20).into_font());
            },
        ));

        chart
            .draw_series(LineSeries::new(
                results.iter().map(|v| (v.x as f32, v.y as f32)),
                &plotters::style::RED,
            ))
            .unwrap();

        root.present().unwrap();
    }
}
