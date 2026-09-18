use nalgebra::{DMatrix, SymmetricEigen};

pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

pub fn pca_2d(embeddings: &[Vec<f32>]) -> Vec<Point2D> {
    assert!(!embeddings.is_empty());

    let n = embeddings.len();
    let dimensions = embeddings[0].len();

    assert!(dimensions >= 2);

    // Make sure all embeddings have the same dimension.
    assert!(
        embeddings
            .iter()
            .all(|embedding| embedding.len() == dimensions),
        "all embeddings must have the same dimension"
    );

    // Convert embeddings into a matrix
    let mut matrix = DMatrix::<f32>::zeros(n, dimensions);

    for (row, embedding) in embeddings.iter().enumerate() {
        for (column, value) in embedding.iter().enumerate() {
            matrix[(row, column)] = *value;
        }
    }

    // Center the data
    for column in 0..dimensions {
        let mean = (0..n).map(|row| matrix[(row, column)]).sum::<f32>() / n as f32;

        for row in 0..n {
            matrix[(row, column)] -= mean;
        }
    }

    // Calculate covariance matrix
    let covariance = if n > 1 {
        (&matrix.transpose() * &matrix) / (n as f32 - 1.0)
    } else {
        panic!("PCA requires at least two embeddings");
    };

    // Eigen decomposition
    let eigen = SymmetricEigen::new(covariance);

    // Sort eigenvectors by eigenvalue
    let mut indices: Vec<usize> = (0..dimensions).collect();

    indices.sort_by(|&a, &b| {
        eigen.eigenvalues[b]
            .partial_cmp(&eigen.eigenvalues[a])
            .unwrap()
    });

    // Take the first two principal components
    let pc1 = eigen.eigenvectors.column(indices[0]);
    let pc2 = eigen.eigenvectors.column(indices[1]);

    let components = DMatrix::from_columns(&[pc1.clone_owned(), pc2.clone_owned()]);

    // Project documents into 2D
    let projected = &matrix * components;

    let mut points = Vec::with_capacity(n);

    for row in 0..n {
        points.push(Point2D {
            x: projected[(row, 0)],
            y: projected[(row, 1)],
        });
    }

    points
}
