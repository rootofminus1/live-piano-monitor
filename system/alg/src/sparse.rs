

// TODO: look up LARS or OMP crates instead of doing all this (prob has a hidden bug somewhere or not who knows)

pub fn sparse_encode(signal: &[f32], dict: &[Vec<f32>], n_nonzero: usize) -> Vec<f32> {
    let n_atoms = dict.len();
    let mut coeffs = vec![0.0f32; n_atoms];
    let mut residual = signal.to_vec();
    let mut selected: Vec<usize> = Vec::with_capacity(n_nonzero);

    for _ in 0..n_nonzero {
        let best = (0..n_atoms)
            .filter(|i| !selected.contains(i))
            .map(|i| {
                let dot: f32 = dict[i].iter().zip(residual.iter()).map(|(a, b)| a * b).sum();
                (i, dot.abs())
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let Some((idx, _)) = best else { break };
        selected.push(idx);
        let k = selected.len();
        let sub: Vec<&Vec<f32>> = selected.iter().map(|&i| &dict[i]).collect();

        let mut gram = vec![0.0f32; k * k];
        for i in 0..k {
            for j in 0..k {
                gram[i * k + j] = sub[i].iter().zip(sub[j].iter()).map(|(a, b)| a * b).sum();
            }
        }

        let rhs: Vec<f32> = sub.iter()
            .map(|atom| atom.iter().zip(signal.iter()).map(|(a, b)| a * b).sum())
            .collect();

        let c = ldl_solve(&gram, k, &rhs);

        residual = signal.to_vec();
        for (ci, &si) in c.iter().zip(selected.iter()) {
            for (r, &a) in residual.iter_mut().zip(dict[si].iter()) { *r -= ci * a; }
        }

        coeffs = vec![0.0f32; n_atoms];
        for (ci, &si) in c.iter().zip(selected.iter()) { 
            coeffs[si] = *ci; 
        }
    }
    coeffs
}

fn ldl_solve(a: &[f32], k: usize, b: &[f32]) -> Vec<f32> {
    let mut l = vec![0.0f32; k * k];
    let mut d = vec![0.0f32; k];

    for i in 0..k {
        let mut diag = a[i * k + i];
        for p in 0..i { diag -= l[i * k + p] * l[i * k + p] * d[p]; }
        d[i] = diag;
        l[i * k + i] = 1.0;

        for j in (i + 1)..k {
            let mut val = a[j * k + i];
            for p in 0..i { val -= l[j * k + p] * l[i * k + p] * d[p]; }
            l[j * k + i] = if diag.abs() > 1e-12 { val / diag } else { 0.0 };
        }
    }

    let mut y = vec![0.0f32; k];

    for i in 0..k {
        y[i] = b[i];
        for p in 0..i { y[i] -= l[i * k + p] * y[p]; }
    }

    let z: Vec<f32> = y.iter().zip(d.iter())
        .map(|(&yi, &di)| if di.abs() > 1e-12 { yi / di } else { 0.0 })
        .collect();

    let mut x = vec![0.0f32; k];

    for i in (0..k).rev() {
        x[i] = z[i];
        for p in (i + 1)..k { x[i] -= l[p * k + i] * x[p]; }
    }
    
    x
}