use nalgebra as na;

#[allow(dead_code)]
#[allow(unused_variables)]
fn main() {
    /* Matrix Type */

    // Statically sized and statically allocated 2x3 matrix using 32-bit floats.
    type Matrix2x3f = na::SMatrix<f32, 2, 3>;

    // Half-dynamically sized and dynamically allocated matrix with
    // two rows using 64-bit floats.
    //
    // The `OMatrix` alias is a matrix that owns its data (as opposed to
    // matrix view which borrow their data from another matrix).
    type Matrix2xXf64 = na::OMatrix<f64, na::U2, na::Dyn>;

    // Dynamically sized and dynamically allocated matrix with two rows
    // and using 32-bit signed integers.
    type DMatrixi32 = na::DMatrix<i32>;
    // or
    type DMatrixi32_2 = na::OMatrix<i32, na::Dyn, na::Dyn>;

    /* Matrix & Vertex construction */

    let v_1 = na::Vector3::new(1, 1, 0);
    let v_x = na::Vector3::<f32>::x();
    let v_x_axis = na::Vector3::<f32>::x_axis();
    assert_eq!(v_x, v_x_axis.into_inner());

    let rv_1 = na::RowVector3::new(1, 1, 0);

    let m_1 = na::Matrix1x2::new(1, 2);
    let m_2 = na::Matrix2x3::<i32>::zeros();
    let mut m_3 = na::Matrix2x3::<i32>::identity();
    assert_eq!(m_3, na::Matrix2x3::new(1, 0, 0, 0, 1, 0));

    /* Matrix element modification */
    m_3[(1, 2)] = 1;
    assert_eq!(m_3, na::Matrix2x3::new(1, 0, 0, 0, 1, 1));

    /* Points */

    // Build using components directly.
    let p_origin = na::Point3::origin();
    let p0 = na::Point3::new(2.0, 3.0, 4.0);

    // Build by translating the origin.
    let translation = na::Vector3::new(2.0, 3.0, 4.0);
    let p2 = p_origin + translation;

    // Build from a coordinates vector.
    let coords = na::Vector3::new(2.0, 3.0, 4.0);
    let p1 = na::Point3::from(coords);

    // Build from homogeneous coordinates. The last component of the
    // vector will be removed and all other components divided by 10.0.
    let homogeneous_coords = na::Vector4::new(20.0, 30.0, 40.0, 10.0);
    let p3 = na::Point3::from_homogeneous(homogeneous_coords).unwrap();

    assert_eq!(p0, p1);
    assert_eq!(p0, p2);
    assert_eq!(p0, p3);

    /* Transformations of points */
}
