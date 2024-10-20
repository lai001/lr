use crate::url_extension::UrlExtension;
use rs_artifact::{asset::Asset, resource_type::EResourceType};
use serde::{Deserialize, Serialize};
use uniform_cubic_splines::{basis::CatmullRom, spline, spline_inverse};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ControlPoint {
    pub position: glam::DVec2,
    pub id: String,
}

impl ControlPoint {
    pub fn new(index: usize, pos: glam::DVec2) -> ControlPoint {
        ControlPoint {
            position: pos,
            id: format!("ControlPoint_{}", index),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Curve {
    pub url: url::Url,
    pub control_points: Vec<ControlPoint>,
}

impl Curve {
    pub fn new(url: url::Url) -> Curve {
        Curve {
            url,
            control_points: vec![],
        }
    }

    pub fn get_name(&self) -> String {
        self.url.get_name_in_editor()
    }

    pub fn sort_by_x(&mut self) {
        self.control_points
            .sort_by(|a, b| a.position.x.total_cmp(&b.position.x));
    }

    pub fn evaluate(&self, x: f64) -> Option<f64> {
        let control_points = &self.control_points;
        if control_points.len() < 2 {
            return None;
        }
        let mut knot_spacing: Vec<f64> = Vec::with_capacity(2 + control_points.len());
        knot_spacing.push(control_points[0].position.x);
        for item in control_points {
            knot_spacing.push(item.position.x);
        }
        knot_spacing.push(control_points.last().unwrap().position.x);

        let mut knots: Vec<f64> = Vec::with_capacity(2 + control_points.len());
        knots.push(control_points[0].position.y);
        for item in control_points {
            knots.push(item.position.y);
        }
        knots.push(control_points.last().unwrap().position.y);
        let v = spline_inverse::<CatmullRom, _>(x, &knot_spacing, None, None).unwrap();
        let y = spline::<CatmullRom, _, _>(v, &knots);
        Some(y)
    }

    pub fn get_x_range(&self) -> Option<std::ops::RangeInclusive<f64>> {
        let control_points = &self.control_points;
        if control_points.len() < 2 {
            return None;
        }
        let min_value = control_points
            .iter()
            .min_by(|lhs, rhs| lhs.position.x.total_cmp(&rhs.position.x))
            .map(|x| x.position.x);
        let max_value = control_points
            .iter()
            .max_by(|lhs, rhs| lhs.position.x.total_cmp(&rhs.position.x))
            .map(|x| x.position.x);

        if let (Some(min_value), Some(max_value)) = (min_value, max_value) {
            Some(min_value..=max_value)
        } else {
            None
        }
    }
}

impl Asset for Curve {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Content(rs_artifact::content_type::EContentType::Curve)
    }
}
