use crate::atoms::{Atoms, Lattice};
use crate::utils;
use std::collections::HashMap;
use std::ops::Index;

/// Structure for managing the reference density and movement within it
/// > data: [f64; Size.total()] - charge density in a flattened array
/// > lattice: Lattice - information on the cell
/// > index: Vec<usize> - sorted list of indices for accessing the data
/// > shift: Shift - contains the values for moving arounf the array
/// > size: Size - the 3d size of the data
/// > voxel_lattice: Lattice - information on the voxel basis
pub struct Density<'a> {
    pub data: &'a Vec<f64>,
    shift: Shift,
    pub size: Size,
    pub vacuum_tolerance: Option<f64>,
    pub voxel_lattice: Lattice,
    pub voxel_origin: [f64; 3],
}

impl<'a> Index<isize> for Density<'a> {
    type Output = f64;

    /// Index the reference charge inside the Density structure
    fn index(&self, i: isize) -> &Self::Output {
        return &self.data[i as usize];
    }
}

impl<'a> Density<'a> {
    /// Initialises a density structure. Computes the voxel_lattice from the grid and lattice.
    pub fn new(data: &'a Vec<f64>,
               grid: [usize; 3],
               lattice: [[f64; 3]; 3],
               vacuum_tolerance: Option<f64>,
               voxel_origin: [f64; 3])
               -> Self {
        let size = Size::new(grid[0], grid[1], grid[2]);
        assert_eq!(size.total, data.len());
        let shift = Shift::new(&size);
        let voxel_lattice = Lattice::new([[lattice[0][0] / grid[0] as f64,
                                           lattice[0][1] / grid[0] as f64,
                                           lattice[0][2] / grid[0] as f64],
                                          [lattice[1][0] / grid[1] as f64,
                                           lattice[1][1] / grid[1] as f64,
                                           lattice[1][2] / grid[1] as f64],
                                          [lattice[2][0] / grid[2] as f64,
                                           lattice[2][1] / grid[2] as f64,
                                           lattice[2][2] / grid[2] as f64]]);
        Self { data,
               shift,
               size,
               vacuum_tolerance,
               voxel_lattice,
               voxel_origin }
    }

    /// get the full shift to visit the surrounding 26 voxels
    pub fn full_shift(&self, p: isize) -> [isize; 26] {
        let shift = self.shift.get(p);
        return [shift[0], shift[1], shift[2], shift[3], shift[4], shift[5],
                shift[6], shift[7], shift[8], shift[9], shift[10], shift[11],
                shift[12], shift[14], shift[15], shift[16], shift[17],
                shift[18], shift[19], shift[20], shift[21], shift[22],
                shift[23], shift[24], shift[25], shift[26]];
    }

    /// get the reduced shift to visit the surrounding 6 voxels
    pub fn reduced_shift(&self, p: isize) -> [isize; 6] {
        let shift = self.shift.get(p);
        // [ +x, -x, +y, -y, +z, -z ]
        return [shift[22], shift[4], shift[16], shift[10], shift[14],
                shift[12]];
    }

    /// Get the gradient shift from the current point
    pub fn gradient_shift(&self, p: isize, gradient: [f64; 3]) -> isize {
        let shift = self.shift.get(p);
        // [ [-x, 0, +x], [-y, 0, +y], [-z, 0, +z] ]
        let i = (gradient[0] * 9f64 + gradient[1] * 3f64 + gradient[2] + 13f64)
                as usize;
        return shift[i];
    }
}

/// Structure for holding the periodic shifts
struct Shift {
    di: [[isize; 27]; 27],
    index: Vec<u8>,
}

impl Shift {
    /// Gets the periodic boundary shift
    fn get(&self, i: isize) -> [isize; 27] {
        let ii = self.index[i as usize] as usize;
        return self.di[ii];
    }

    /// Generate the indices for Shift.index
    fn index_gen(size: &Size) -> Vec<u8> {
        let mut index = vec![0u8; size.total];
        let mut i: usize = 0;
        let bound_x: usize = (size.x - 1) as usize;
        let bound_y: usize = (size.y - 1) as usize;
        let bound_z: usize = (size.z - 1) as usize;

        // first x
        // first y
        index[i] = 26;
        i += 1;
        for _ in 1..bound_z {
            index[i] = 24;
            i += 1;
        }
        index[i] = 25;
        i += 1;
        // bulk y
        for _ in 1..bound_y {
            index[i] = 20;
            i += 1;
            for _ in 1..bound_z {
                index[i] = 18;
                i += 1;
            }
            index[i] = 19;
            i += 1;
        }
        // last y
        index[i] = 23;
        i += 1;
        for _ in 1..bound_z {
            index[i] = 21;
            i += 1;
        }
        index[i] = 22;
        i += 1;

        // bulk x
        // first y
        for _ in 1..bound_x {
            index[i] = 8;
            i += 1;
            for _ in 1..bound_z {
                index[i] = 6;
                i += 1;
            }
            index[i] = 7;
            i += 1;
            // bulk y
            for _ in 1..bound_y {
                index[i] = 2;
                i += bound_z;
                index[i] = 1;
                i += 1;
            }
            // last y
            index[i] = 5;
            i += 1;
            for _ in 1..bound_z {
                index[i] = 3;
                i += 1;
            }
            index[i] = 4;
            i += 1;
        }

        // last x
        // first y
        index[i] = 17;
        i += 1;
        for _ in 1..bound_z {
            index[i] = 15;
            i += 1;
        }
        index[i] = 16;
        i += 1;
        // bulk y
        for _ in 1..bound_y {
            index[i] = 11;
            i += 1;
            for _ in 1..bound_z {
                index[i] = 9;
                i += 1;
            }
            index[i] = 10;
            i += 1;
        }
        // last y
        index[i] = 14;
        i += 1;
        for _ in 1..bound_z {
            index[i] = 12;
            i += 1;
        }
        index[i] = 13;
        return index;
    }

    /// Creates the shift matrix for moving in the array
    /// the outer index in di is unraveling the index with a periodic roll so the point
    /// is at (0, 0, 0) -> 0
    /// inside is the same as the distance array in Density
    /// > 0 -> (-1,-1,-1)   7 -> (-1, 1, 0)  14 -> (0, 0, 1)  21 -> (1, 0,-1)
    /// > 1 -> (-1,-1, 0)   8 -> (-1, 1, 1)  15 -> (0, 1,-1)  22 -> (1, 0, 0)
    /// > 2 -> (-1,-1, 1)   9 -> (0,-1,-1)   16 -> (0, 1, 0)  23 -> (1, 0, 1)
    /// > 3 -> (-1, 0,-1)  10 -> (0,-1, 0)   17 -> (0, 1, 1)  24 -> (1, 1,-1)
    /// > 4 -> (-1, 0, 0)  11 -> (0,-1, 1)   18 -> (1,-1,-1)  25 -> (1, 1, 0)
    /// > 5 -> (-1, 0, 1)  12 -> (0, 0,-1)   19 -> (1,-1, 0)  26 -> (1, 1, 1)
    /// > 6 -> (-1, 1,-1)  13 -> (0, 0, 0)   20 -> (1,-1, 1)
    /// BEWARE: The code for this is god awful
    fn new(size: &Size) -> Self {
        let mut di = [[0isize; 27]; 27];
        let index = Shift::index_gen(size);

        let _x: isize = -1 * (size.y * size.z);
        let _xx: isize = (size.x * size.y * size.z) + _x;
        let x: isize = size.y * size.z;
        let xx: isize = -1 * (size.x * size.y * size.z) + x;
        let _y: isize = -1 * size.z;
        let _yy: isize = (size.y * size.z) + _y;
        let y: isize = size.z;
        let yy: isize = -1 * (size.y * size.z) + y;
        let _z: isize = -1;
        let _zz: isize = size.z + _z;
        let z: isize = 1;
        let zz: isize = (-1 * size.z) + z;

        // GET READY FOR SOME FRESH HELL VECTOR CREATION
        // We need to make the 27 index shifts for the 27 different positions
        // we can be in the density grid

        // (0, 0, 0)
        di[0] = [_x + _y + _z,
                 _x + _y,
                 _x + _y + z,
                 _x + _z,
                 _x,
                 _x + z,
                 _x + y + _z,
                 _x + y,
                 _x + y + z,
                 _y + _z,
                 _y,
                 _y + z,
                 _z,
                 0,
                 z,
                 y + _z,
                 y,
                 y + z,
                 x + _y + _z,
                 x + _y,
                 x + _y + z,
                 x + _z,
                 x,
                 x + z,
                 x + y + _z,
                 x + y,
                 x + y + z];
        // (0, 0, 1)
        di[1] = [_x + _y + _z,
                 _x + _y,
                 _x + _y + zz,
                 _x + _z,
                 _x,
                 _x + zz,
                 _x + y + _z,
                 _x + y,
                 _x + y + zz,
                 _y + _z,
                 _y,
                 _y + zz,
                 _z,
                 0,
                 zz,
                 y + _z,
                 y,
                 y + zz,
                 x + _y + _z,
                 x + _y,
                 x + _y + zz,
                 x + _z,
                 x,
                 x + zz,
                 x + y + _z,
                 x + y,
                 x + y + zz];
        // (0, 0,-1)
        di[2] = [_x + _y + _zz,
                 _x + _y,
                 _x + _y + z,
                 _x + _zz,
                 _x,
                 _x + z,
                 _x + y + _zz,
                 _x + y,
                 _x + y + z,
                 _y + _zz,
                 _y,
                 _y + z,
                 _zz,
                 0,
                 z,
                 y + _zz,
                 y,
                 y + z,
                 x + _y + _zz,
                 x + _y,
                 x + _y + z,
                 x + _zz,
                 x,
                 x + z,
                 x + y + _zz,
                 x + y,
                 x + y + z];
        // (0, 1, 0)
        di[3] = [_x + _y + _z,
                 _x + _y,
                 _x + _y + z,
                 _x + _z,
                 _x,
                 _x + z,
                 _x + yy + _z,
                 _x + yy,
                 _x + yy + z,
                 _y + _z,
                 _y,
                 _y + z,
                 _z,
                 0,
                 z,
                 yy + _z,
                 yy,
                 yy + z,
                 x + _y + _z,
                 x + _y,
                 x + _y + z,
                 x + _z,
                 x,
                 x + z,
                 x + yy + _z,
                 x + yy,
                 x + yy + z];
        // (0, 1, 1)
        di[4] = [_x + _y + _z,
                 _x + _y,
                 _x + _y + zz,
                 _x + _z,
                 _x,
                 _x + zz,
                 _x + yy + _z,
                 _x + yy,
                 _x + yy + zz,
                 _y + _z,
                 _y,
                 _y + zz,
                 _z,
                 0,
                 zz,
                 yy + _z,
                 yy,
                 yy + zz,
                 x + _y + _z,
                 x + _y,
                 x + _y + zz,
                 x + _z,
                 x,
                 x + zz,
                 x + yy + _z,
                 x + yy,
                 x + yy + zz];
        // (0, 1,-1)
        di[5] = [_x + _y + _zz,
                 _x + _y,
                 _x + _y + z,
                 _x + _zz,
                 _x,
                 _x + z,
                 _x + yy + _zz,
                 _x + yy,
                 _x + yy + z,
                 _y + _zz,
                 _y,
                 _y + z,
                 _zz,
                 0,
                 z,
                 yy + _zz,
                 yy,
                 yy + z,
                 x + _y + _zz,
                 x + _y,
                 x + _y + z,
                 x + _zz,
                 x,
                 x + z,
                 x + yy + _zz,
                 x + yy,
                 x + yy + z];
        // (0,-1, 0)
        di[6] = [_x + _yy + _z,
                 _x + _yy,
                 _x + _yy + z,
                 _x + _z,
                 _x,
                 _x + z,
                 _x + y + _z,
                 _x + y,
                 _x + y + z,
                 _yy + _z,
                 _yy,
                 _yy + z,
                 _z,
                 0,
                 z,
                 y + _z,
                 y,
                 y + z,
                 x + _yy + _z,
                 x + _yy,
                 x + _yy + z,
                 x + _z,
                 x,
                 x + z,
                 x + y + _z,
                 x + y,
                 x + y + z];
        // (0,-1, 1)
        di[7] = [_x + _yy + _z,
                 _x + _yy,
                 _x + _yy + zz,
                 _x + _z,
                 _x,
                 _x + zz,
                 _x + y + _z,
                 _x + y,
                 _x + y + zz,
                 _yy + _z,
                 _yy,
                 _yy + zz,
                 _z,
                 0,
                 zz,
                 y + _z,
                 y,
                 y + zz,
                 x + _yy + _z,
                 x + _yy,
                 x + _yy + zz,
                 x + _z,
                 x,
                 x + zz,
                 x + y + _z,
                 x + y,
                 x + y + zz];
        // (0,-1,-1)
        di[8] = [_x + _yy + _zz,
                 _x + _yy,
                 _x + _yy + z,
                 _x + _zz,
                 _x,
                 _x + z,
                 _x + y + _zz,
                 _x + y,
                 _x + y + z,
                 _yy + _zz,
                 _yy,
                 _yy + z,
                 _zz,
                 0,
                 z,
                 y + _zz,
                 y,
                 y + z,
                 x + _yy + _zz,
                 x + _yy,
                 x + _yy + z,
                 x + _zz,
                 x,
                 x + z,
                 x + y + _zz,
                 x + y,
                 x + y + z];
        // (1, 0, 0)
        di[9] = [_x + _y + _z,
                 _x + _y,
                 _x + _y + z,
                 _x + _z,
                 _x,
                 _x + z,
                 _x + y + _z,
                 _x + y,
                 _x + y + z,
                 _y + _z,
                 _y,
                 _y + z,
                 _z,
                 0,
                 z,
                 y + _z,
                 y,
                 y + z,
                 xx + _y + _z,
                 xx + _y,
                 xx + _y + z,
                 xx + _z,
                 xx,
                 xx + z,
                 xx + y + _z,
                 xx + y,
                 xx + y + z];
        // (1, 0, 1)
        di[10] = [_x + _y + _z,
                  _x + _y,
                  _x + _y + zz,
                  _x + _z,
                  _x,
                  _x + zz,
                  _x + y + _z,
                  _x + y,
                  _x + y + zz,
                  _y + _z,
                  _y,
                  _y + zz,
                  _z,
                  0,
                  zz,
                  y + _z,
                  y,
                  y + zz,
                  xx + _y + _z,
                  xx + _y,
                  xx + _y + zz,
                  xx + _z,
                  xx,
                  xx + zz,
                  xx + y + _z,
                  xx + y,
                  xx + y + zz];
        // (1, 0,-1)
        di[11] = [_x + _y + _zz,
                  _x + _y,
                  _x + _y + z,
                  _x + _zz,
                  _x,
                  _x + z,
                  _x + y + _zz,
                  _x + y,
                  _x + y + z,
                  _y + _zz,
                  _y,
                  _y + z,
                  _zz,
                  0,
                  z,
                  y + _zz,
                  y,
                  y + z,
                  xx + _y + _zz,
                  xx + _y,
                  xx + _y + z,
                  xx + _zz,
                  xx,
                  xx + z,
                  xx + y + _zz,
                  xx + y,
                  xx + y + z];
        // (1, 1, 0)
        di[12] = [_x + _y + _z,
                  _x + _y,
                  _x + _y + z,
                  _x + _z,
                  _x,
                  _x + z,
                  _x + yy + _z,
                  _x + yy,
                  _x + yy + z,
                  _y + _z,
                  _y,
                  _y + z,
                  _z,
                  0,
                  z,
                  yy + _z,
                  yy,
                  yy + z,
                  xx + _y + _z,
                  xx + _y,
                  xx + _y + z,
                  xx + _z,
                  xx,
                  xx + z,
                  xx + yy + _z,
                  xx + yy,
                  xx + yy + z];
        // (1, 1, 1)
        di[13] = [_x + _y + _z,
                  _x + _y,
                  _x + _y + zz,
                  _x + _z,
                  _x,
                  _x + zz,
                  _x + yy + _z,
                  _x + yy,
                  _x + yy + zz,
                  _y + _z,
                  _y,
                  _y + zz,
                  _z,
                  0,
                  zz,
                  yy + _z,
                  yy,
                  yy + zz,
                  xx + _y + _z,
                  xx + _y,
                  xx + _y + zz,
                  xx + _z,
                  xx,
                  xx + zz,
                  xx + yy + _z,
                  xx + yy,
                  xx + yy + zz];
        // (1, 1,-1)
        di[14] = [_x + _y + _zz,
                  _x + _y,
                  _x + _y + z,
                  _x + _zz,
                  _x,
                  _x + z,
                  _x + yy + _zz,
                  _x + yy,
                  _x + yy + z,
                  _y + _zz,
                  _y,
                  _y + z,
                  _zz,
                  0,
                  z,
                  yy + _zz,
                  yy,
                  yy + z,
                  xx + _y + _zz,
                  xx + _y,
                  xx + _y + z,
                  xx + _zz,
                  xx,
                  xx + z,
                  xx + yy + _zz,
                  xx + yy,
                  xx + yy + z];
        // (1,-1, 0)
        di[15] = [_x + _yy + _z,
                  _x + _yy,
                  _x + _yy + z,
                  _x + _z,
                  _x,
                  _x + z,
                  _x + y + _z,
                  _x + y,
                  _x + y + z,
                  _yy + _z,
                  _yy,
                  _yy + z,
                  _z,
                  0,
                  z,
                  y + _z,
                  y,
                  y + z,
                  xx + _yy + _z,
                  xx + _yy,
                  xx + _yy + z,
                  xx + _z,
                  xx,
                  xx + z,
                  xx + y + _z,
                  xx + y,
                  xx + y + z];
        // (1,-1, 1)
        di[16] = [_x + _yy + _z,
                  _x + _yy,
                  _x + _yy + zz,
                  _x + _z,
                  _x,
                  _x + zz,
                  _x + y + _z,
                  _x + y,
                  _x + y + zz,
                  _yy + _z,
                  _yy,
                  _yy + zz,
                  _z,
                  0,
                  zz,
                  y + _z,
                  y,
                  y + zz,
                  xx + _yy + _z,
                  xx + _yy,
                  xx + _yy + zz,
                  xx + _z,
                  xx,
                  xx + zz,
                  xx + y + _z,
                  xx + y,
                  xx + y + zz];
        // (1,-1,-1)
        di[17] = [_x + _yy + _zz,
                  _x + _yy,
                  _x + _yy + z,
                  _x + _zz,
                  _x,
                  _x + z,
                  _x + y + _zz,
                  _x + y,
                  _x + y + z,
                  _yy + _zz,
                  _yy,
                  _yy + z,
                  _zz,
                  0,
                  z,
                  y + _zz,
                  y,
                  y + z,
                  xx + _yy + _zz,
                  xx + _yy,
                  xx + _yy + z,
                  xx + _zz,
                  xx,
                  xx + z,
                  xx + y + _zz,
                  xx + y,
                  xx + y + z];
        // (-1, 0, 0)
        di[18] = [_xx + _y + _z,
                  _xx + _y,
                  _xx + _y + z,
                  _xx + _z,
                  _xx,
                  _xx + z,
                  _xx + y + _z,
                  _xx + y,
                  _xx + y + z,
                  _y + _z,
                  _y,
                  _y + z,
                  _z,
                  0,
                  z,
                  y + _z,
                  y,
                  y + z,
                  x + _y + _z,
                  x + _y,
                  x + _y + z,
                  x + _z,
                  x,
                  x + z,
                  x + y + _z,
                  x + y,
                  x + y + z];
        // (-1, 0, 1)
        di[19] = [_xx + _y + _z,
                  _xx + _y,
                  _xx + _y + zz,
                  _xx + _z,
                  _xx,
                  _xx + zz,
                  _xx + y + _z,
                  _xx + y,
                  _xx + y + zz,
                  _y + _z,
                  _y,
                  _y + zz,
                  _z,
                  0,
                  zz,
                  y + _z,
                  y,
                  y + zz,
                  x + _y + _z,
                  x + _y,
                  x + _y + zz,
                  x + _z,
                  x,
                  x + zz,
                  x + y + _z,
                  x + y,
                  x + y + zz];
        // (-1, 0,-1)
        di[20] = [_xx + _y + _zz,
                  _xx + _y,
                  _xx + _y + z,
                  _xx + _zz,
                  _xx,
                  _xx + z,
                  _xx + y + _zz,
                  _xx + y,
                  _xx + y + z,
                  _y + _zz,
                  _y,
                  _y + z,
                  _zz,
                  0,
                  z,
                  y + _zz,
                  y,
                  y + z,
                  x + _y + _zz,
                  x + _y,
                  x + _y + z,
                  x + _zz,
                  x,
                  x + z,
                  x + y + _zz,
                  x + y,
                  x + y + z];
        // (-1, 1, 0)
        di[21] = [_xx + _y + _z,
                  _xx + _y,
                  _xx + _y + z,
                  _xx + _z,
                  _xx,
                  _xx + z,
                  _xx + yy + _z,
                  _xx + yy,
                  _xx + yy + z,
                  _y + _z,
                  _y,
                  _y + z,
                  _z,
                  0,
                  z,
                  yy + _z,
                  yy,
                  yy + z,
                  x + _y + _z,
                  x + _y,
                  x + _y + z,
                  x + _z,
                  x,
                  x + z,
                  x + yy + _z,
                  x + yy,
                  x + yy + z];
        // (-1, 1, 1)
        di[22] = [_xx + _y + _z,
                  _xx + _y,
                  _xx + _y + zz,
                  _xx + _z,
                  _xx,
                  _xx + zz,
                  _xx + yy + _z,
                  _xx + yy,
                  _xx + yy + zz,
                  _y + _z,
                  _y,
                  _y + zz,
                  _z,
                  0,
                  zz,
                  yy + _z,
                  yy,
                  yy + zz,
                  x + _y + _z,
                  x + _y,
                  x + _y + zz,
                  x + _z,
                  x,
                  x + zz,
                  x + yy + _z,
                  x + yy,
                  x + yy + zz];
        // (-1, 1,-1)
        di[23] = [_xx + _y + _zz,
                  _xx + _y,
                  _xx + _y + z,
                  _xx + _zz,
                  _xx,
                  _xx + z,
                  _xx + yy + _zz,
                  _xx + yy,
                  _xx + yy + z,
                  _y + _zz,
                  _y,
                  _y + z,
                  _zz,
                  0,
                  z,
                  yy + _zz,
                  yy,
                  yy + z,
                  x + _y + _zz,
                  x + _y,
                  x + _y + z,
                  x + _zz,
                  x,
                  x + z,
                  x + yy + _zz,
                  x + yy,
                  x + yy + z];
        // (-1,-1, 0)
        di[24] = [_xx + _yy + _z,
                  _xx + _yy,
                  _xx + _yy + z,
                  _xx + _z,
                  _xx,
                  _xx + z,
                  _xx + y + _z,
                  _xx + y,
                  _xx + y + z,
                  _yy + _z,
                  _yy,
                  _yy + z,
                  _z,
                  0,
                  z,
                  y + _z,
                  y,
                  y + z,
                  x + _yy + _z,
                  x + _yy,
                  x + _yy + z,
                  x + _z,
                  x,
                  x + z,
                  x + y + _z,
                  x + y,
                  x + y + z];
        // (-1,-1, 1)
        di[25] = [_xx + _yy + _z,
                  _xx + _yy,
                  _xx + _yy + zz,
                  _xx + _z,
                  _xx,
                  _xx + zz,
                  _xx + y + _z,
                  _xx + y,
                  _xx + y + zz,
                  _yy + _z,
                  _yy,
                  _yy + zz,
                  _z,
                  0,
                  zz,
                  y + _z,
                  y,
                  y + zz,
                  x + _yy + _z,
                  x + _yy,
                  x + _yy + zz,
                  x + _z,
                  x,
                  x + zz,
                  x + y + _z,
                  x + y,
                  x + y + zz];
        // (-1,-1,-1)
        di[26] = [_xx + _yy + _zz,
                  _xx + _yy,
                  _xx + _yy + z,
                  _xx + _zz,
                  _xx,
                  _xx + z,
                  _xx + y + _zz,
                  _xx + y,
                  _xx + y + z,
                  _yy + _zz,
                  _yy,
                  _yy + z,
                  _zz,
                  0,
                  z,
                  y + _zz,
                  y,
                  y + z,
                  x + _yy + _zz,
                  x + _yy,
                  x + _yy + z,
                  x + _zz,
                  x,
                  x + z,
                  x + y + _zz,
                  x + y,
                  x + y + z];
        return Self { di, index };
    }
}

/// Size of the density data in 3d
pub struct Size {
    pub x: isize,
    pub y: isize,
    pub z: isize,
    pub total: usize,
}

impl Size {
    /// The length of the flattened array for the density data in 3d
    fn new(x: usize, y: usize, z: usize) -> Self {
        let x = x as isize;
        let y = y as isize;
        let z = z as isize;
        let total = match x.checked_mul(y) {
            Some(xy) => match xy.checked_mul(z) {
                Some(xyz) => xyz as usize,
                None => panic!("Grid size is too large!"),
            },
            None => panic!("Grid size is too large!"),
        };
        return Self { x, y, z, total };
    }
}

/// VoxelMap - enum for mapping the voxels to bader volumes
///
/// > Just(Vec<usize>) - Just the voxel map (ongrid)
/// > Known { map: Vec<usize>, known: Vec<bool> } - Holds the map and a known
/// >                                               array
pub struct VoxelMap {
    pub map: Vec<isize>,
    pub bader_maxima: Vec<isize>,
    index: HashMap<isize, usize>,
}

impl VoxelMap {
    /// Initialise the structure. Create a HashMap to index the volumes.
    pub fn new(map: Vec<isize>, bader_maxima: Vec<isize>) -> Self {
        let index = if bader_maxima[0] < 0 {
            bader_maxima[1..].iter()
                             .enumerate()
                             .map(|(i, v)| (*v, i))
                             .collect::<HashMap<isize, usize>>()
        } else {
            bader_maxima[..].iter()
                            .enumerate()
                            .map(|(i, v)| (*v, i))
                            .collect::<HashMap<isize, usize>>()
        };
        return Self { map,
                      bader_maxima,
                      index };
    }

    /// Is the point known, use shifts to move around
    pub fn is_known(&self, p: usize, shifts: [isize; 6]) -> bool {
        for shift in shifts.iter() {
            if self.map[p] != self.map[(p as isize + shift) as usize] {
                return false;
            }
        }
        return true;
    }

    /// Sum the densities for each bader volume
    pub fn charge_sum(&self,
                      densities: &Vec<Vec<f64>>)
                      -> (Vec<Vec<f64>>, Vec<usize>, f64, usize) {
        let mut vacuum_charge = 0f64;
        let mut vacuum_volume = 0usize;
        let mut bader_volume = vec![0usize; self.index.len()];
        let mut bader_charge =
            vec![vec![0f64; self.index.len()]; densities.len()];
        for (i, maxima) in self.map.iter().enumerate() {
            if *maxima < 0 {
                vacuum_charge += densities[0][i];
                vacuum_volume += 1;
            } else {
                bader_volume[self.index[maxima]] += 1;
                for j in 0..densities.len() {
                    bader_charge[j][self.index[maxima]] += densities[j][i];
                }
            }
        }
        return (bader_charge, bader_volume, vacuum_charge, vacuum_volume);
    }

    /// Calculates the minimum distance to the surface from the bader maxima
    pub fn surface_distance(&self,
                            assigned_atom: &Vec<usize>,
                            atoms: &Atoms,
                            density: &Density)
                            -> Vec<f64> {
        // keep two distance vectors as atoms without volume have zero distance
        let mut surface_distance = vec![0f64; atoms.positions.len()];
        let mut minimum_distance = {
            vec![
                atoms.lattice.distance_matrix[0].powi(2);
                atoms.positions.len()
            ]
        };
        for (p, maxima) in self.map.iter().enumerate() {
            if *maxima < 0 {
                continue;
            }
            if self.is_known(p, density.reduced_shift(p as isize)) {
                continue;
            }
            let px = density.voxel_origin[0]
                     + (p as isize / (density.size.y * density.size.z)) as f64;
            let py = density.voxel_origin[1]
                     + (p as isize / density.size.z).rem_euclid(density.size.y)
                       as f64;
            let pz = density.voxel_origin[2]
                     + (p as isize).rem_euclid(density.size.z) as f64;
            let p_cartesian =
                utils::dot([px, py, pz], density.voxel_lattice.to_cartesian);
            let atom_num = assigned_atom[self.index[maxima]];
            let atom = atoms.positions[atom_num];
            for atom_shift in atoms.lattice.shift_matrix.iter() {
                let distance = {
                    (p_cartesian[0] - (atom[0] + atom_shift[0])).powi(2)
                    + (p_cartesian[1] - (atom[1] + atom_shift[1])).powi(2)
                    + (p_cartesian[2] - (atom[2] + atom_shift[2])).powi(2)
                };
                if distance < minimum_distance[atom_num] {
                    surface_distance[atom_num] = distance.powf(0.5);
                    minimum_distance[atom_num] = distance;
                }
            }
        }
        return surface_distance;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_new() {
        let data = (0..64).map(|x| x as f64).collect::<Vec<f64>>();
        let lattice = Lattice::new([[3., 3., 0.], [-3., 3., 0.], [1., 1., 1.]]);
        let density = Density::new(&data,
                                   [4, 4, 4],
                                   lattice.to_cartesian,
                                   Some(1E-3),
                                   [0., 0., 0.0]);
        assert_eq!(density.voxel_lattice.volume, lattice.volume / 64.)
    }

    #[test]
    #[should_panic]
    fn density_new_bad_grid() {
        let data = (0..64).map(|x| x as f64).collect::<Vec<f64>>();
        let lattice = Lattice::new([[3., 3., 0.], [-3., 3., 0.], [1., 1., 1.]]);
        let _ = Density::new(&data,
                             [1, 4, 4],
                             lattice.to_cartesian,
                             Some(1E-3),
                             [0., 0., 0.0]);
    }

    #[test]
    fn density_full_shift() {
        let data = (0..60).map(|x| x as f64).collect::<Vec<f64>>();
        let lattice = Lattice::new([[3., 3., 0.], [-3., 3., 0.], [1., 1., 1.]]);
        let density = Density::new(&data,
                                   [3, 4, 5],
                                   lattice.to_cartesian,
                                   Some(1E-3),
                                   [0., 0., 0.0]);
        let shift = [-26, -25, -24, -21, -20, -19, -16, -15, -14, -6, -5, -4,
                     -1, 1, 4, 5, 6, 14, 15, 16, 19, 20, 21, 24, 25, 26];
        assert_eq!(shift, density.full_shift(26))
    }

    #[test]
    fn density_reduced_shift() {
        let data = (0..60).map(|x| x as f64).collect::<Vec<f64>>();
        let lattice = Lattice::new([[3., 3., 0.], [-3., 3., 0.], [1., 1., 1.]]);
        let density = Density::new(&data,
                                   [3, 4, 5],
                                   lattice.to_cartesian,
                                   Some(1E-3),
                                   [0., 0., 0.0]);
        let shift = [20, -20, 5, -5, 1, -1];
        assert_eq!(shift, density.reduced_shift(26))
    }

    #[test]
    fn density_gradient_shift() {
        let data = (0..60).map(|x| x as f64).collect::<Vec<f64>>();
        let lattice = Lattice::new([[3., 3., 0.], [-3., 3., 0.], [1., 1., 1.]]);
        let density = Density::new(&data,
                                   [3, 4, 5],
                                   lattice.to_cartesian,
                                   Some(1E-3),
                                   [0., 0., 0.0]);
        assert_eq!(1, density.gradient_shift(26, [0., 0., 1.]))
    }

    #[test]
    fn density_index() {
        let data = (0..60).map(|x| x as f64).collect::<Vec<f64>>();
        let lattice = Lattice::new([[3., 3., 0.], [-3., 3., 0.], [1., 1., 1.]]);
        let density = Density::new(&data,
                                   [3, 4, 5],
                                   lattice.to_cartesian,
                                   Some(1E-3),
                                   [0., 0., 0.0]);
        assert_eq!(&data[1], density.index(1));
        assert_eq!(data[1], density[1]);
    }

    #[test]
    fn shift_index_gen() {
        let index = Shift::index_gen(&Size::new(3, 4, 5));
        let test_index: [usize; 27] = [0, 1, 4, 5, 6, 9, 15, 16, 19, 20, 21,
                                       24, 25, 26, 29, 35, 36, 39, 40, 41, 44,
                                       45, 46, 49, 55, 56, 59];
        let test_value: [u8; 27] = [26, 24, 25, 20, 18, 19, 23, 21, 22, 8, 6,
                                    7, 2, 0, 1, 5, 3, 4, 17, 15, 16, 11, 9,
                                    10, 14, 12, 13];
        for (i, v) in test_index.iter().zip(&test_value) {
            assert_eq!(index[*i], *v)
        }
    }

    #[test]
    fn shift_new() {
        let shift = Shift::new(&Size::new(3, 4, 5));
        let di =
            [[-26, -25, -24, -21, -20, -19, -16, -15, -14, -6, -5, -4, -1, 0,
              1, 4, 5, 6, 14, 15, 16, 19, 20, 21, 24, 25, 26],
             [-26, -25, -29, -21, -20, -24, -16, -15, -19, -6, -5, -9, -1, 0,
              -4, 4, 5, 1, 14, 15, 11, 19, 20, 16, 24, 25, 21],
             [-21, -25, -24, -16, -20, -19, -11, -15, -14, -1, -5, -4, 4, 0,
              1, 9, 5, 6, 19, 15, 16, 24, 20, 21, 29, 25, 26],
             [-26, -25, -24, -21, -20, -19, -36, -35, -34, -6, -5, -4, -1, 0,
              1, -16, -15, -14, 14, 15, 16, 19, 20, 21, 4, 5, 6],
             [-26, -25, -29, -21, -20, -24, -36, -35, -39, -6, -5, -9, -1, 0,
              -4, -16, -15, -19, 14, 15, 11, 19, 20, 16, 4, 5, 1],
             [-21, -25, -24, -16, -20, -19, -31, -35, -34, -1, -5, -4, 4, 0,
              1, -11, -15, -14, 19, 15, 16, 24, 20, 21, 9, 5, 6],
             [-6, -5, -4, -21, -20, -19, -16, -15, -14, 14, 15, 16, -1, 0, 1,
              4, 5, 6, 34, 35, 36, 19, 20, 21, 24, 25, 26],
             [-6, -5, -9, -21, -20, -24, -16, -15, -19, 14, 15, 11, -1, 0,
              -4, 4, 5, 1, 34, 35, 31, 19, 20, 16, 24, 25, 21],
             [-1, -5, -4, -16, -20, -19, -11, -15, -14, 19, 15, 16, 4, 0, 1,
              9, 5, 6, 39, 35, 36, 24, 20, 21, 29, 25, 26],
             [-26, -25, -24, -21, -20, -19, -16, -15, -14, -6, -5, -4, -1, 0,
              1, 4, 5, 6, -46, -45, -44, -41, -40, -39, -36, -35, -34],
             [-26, -25, -29, -21, -20, -24, -16, -15, -19, -6, -5, -9, -1, 0,
              -4, 4, 5, 1, -46, -45, -49, -41, -40, -44, -36, -35, -39],
             [-21, -25, -24, -16, -20, -19, -11, -15, -14, -1, -5, -4, 4, 0,
              1, 9, 5, 6, -41, -45, -44, -36, -40, -39, -31, -35, -34],
             [-26, -25, -24, -21, -20, -19, -36, -35, -34, -6, -5, -4, -1, 0,
              1, -16, -15, -14, -46, -45, -44, -41, -40, -39, -56, -55, -54],
             [-26, -25, -29, -21, -20, -24, -36, -35, -39, -6, -5, -9, -1, 0,
              -4, -16, -15, -19, -46, -45, -49, -41, -40, -44, -56, -55, -59],
             [-21, -25, -24, -16, -20, -19, -31, -35, -34, -1, -5, -4, 4, 0,
              1, -11, -15, -14, -41, -45, -44, -36, -40, -39, -51, -55, -54],
             [-6, -5, -4, -21, -20, -19, -16, -15, -14, 14, 15, 16, -1, 0, 1,
              4, 5, 6, -26, -25, -24, -41, -40, -39, -36, -35, -34],
             [-6, -5, -9, -21, -20, -24, -16, -15, -19, 14, 15, 11, -1, 0,
              -4, 4, 5, 1, -26, -25, -29, -41, -40, -44, -36, -35, -39],
             [-1, -5, -4, -16, -20, -19, -11, -15, -14, 19, 15, 16, 4, 0, 1,
              9, 5, 6, -21, -25, -24, -36, -40, -39, -31, -35, -34],
             [34, 35, 36, 39, 40, 41, 44, 45, 46, -6, -5, -4, -1, 0, 1, 4, 5,
              6, 14, 15, 16, 19, 20, 21, 24, 25, 26],
             [34, 35, 31, 39, 40, 36, 44, 45, 41, -6, -5, -9, -1, 0, -4, 4,
              5, 1, 14, 15, 11, 19, 20, 16, 24, 25, 21],
             [39, 35, 36, 44, 40, 41, 49, 45, 46, -1, -5, -4, 4, 0, 1, 9, 5,
              6, 19, 15, 16, 24, 20, 21, 29, 25, 26],
             [34, 35, 36, 39, 40, 41, 24, 25, 26, -6, -5, -4, -1, 0, 1, -16,
              -15, -14, 14, 15, 16, 19, 20, 21, 4, 5, 6],
             [34, 35, 31, 39, 40, 36, 24, 25, 21, -6, -5, -9, -1, 0, -4, -16,
              -15, -19, 14, 15, 11, 19, 20, 16, 4, 5, 1],
             [39, 35, 36, 44, 40, 41, 29, 25, 26, -1, -5, -4, 4, 0, 1, -11,
              -15, -14, 19, 15, 16, 24, 20, 21, 9, 5, 6],
             [54, 55, 56, 39, 40, 41, 44, 45, 46, 14, 15, 16, -1, 0, 1, 4, 5,
              6, 34, 35, 36, 19, 20, 21, 24, 25, 26],
             [54, 55, 51, 39, 40, 36, 44, 45, 41, 14, 15, 11, -1, 0, -4, 4,
              5, 1, 34, 35, 31, 19, 20, 16, 24, 25, 21],
             [59, 55, 56, 44, 40, 41, 49, 45, 46, 19, 15, 16, 4, 0, 1, 9, 5,
              6, 39, 35, 36, 24, 20, 21, 29, 25, 26]];
        assert_eq!(shift.di, di)
    }

    #[test]
    fn shift_get() {
        assert_eq!(Shift::new(&Size::new(3, 4, 5)).get(25)[0], -21)
    }

    #[test]
    fn size_new() {
        assert_eq!(Size::new(3, 4, 5).total, 3 * 4 * 5)
    }

    #[test]
    #[should_panic]
    #[cfg(target_pointer_width = "32")]
    fn size_too_large_32() {
        let _ = Size::new(1300, 1300, 1300);
    }

    #[test]
    #[should_panic]
    fn size_too_large() {
        let _ = Size::new(2100000, 2100000, 2100000);
    }

    #[test]
    fn voxel_map_new_vacuum() {
        let map: Vec<isize> = vec![1, 1, 1, 2, 2, 1, -1, 2, 2, -1, 2, 1];
        let bader_maxima = vec![-1, 1, 2];
        let voxel_map = VoxelMap::new(map, bader_maxima);
        assert_eq!(voxel_map.index[&1], 0);
        assert_eq!(voxel_map.index[&2], 1);
    }

    #[test]
    fn voxel_map_new_no_vacuum() {
        let map: Vec<isize> = vec![1, 1, 1, 2, 2, 1, 2, 2, 2, 1];
        let bader_maxima = vec![1, 2];
        let voxel_map = VoxelMap::new(map, bader_maxima);
        assert_eq!(voxel_map.index[&1], 0);
        assert_eq!(voxel_map.index[&2], 1);
    }

    #[test]
    fn voxel_map_is_known() {
        let map: Vec<isize> = vec![1, 1, 1, 2, 2, 1, 2, 2, 2, 1, 1, 1];
        let bader_maxima = vec![1, 2];
        let voxel_map = VoxelMap::new(map, bader_maxima);
        assert!(voxel_map.is_known(0, [1, 2, 5, 9, 10, 11]))
    }

    #[test]
    #[should_panic]
    fn voxel_map_not_known() {
        let map: Vec<isize> = vec![1, 1, 1, 2, 2, 1, 2, 2, 2, 1, 1, 1];
        let bader_maxima = vec![1, 2];
        let voxel_map = VoxelMap::new(map, bader_maxima);
        assert!(voxel_map.is_known(0, [1, 2, 5, 8, 10, 11]))
    }

    #[test]
    fn voxel_map_charge_sum() {
        let map: Vec<isize> = vec![1, 1, 1, 2, 2, 1, 2, 2, 2, -1, -1, -1];
        let bader_maxima = vec![-1, 1, 2];
        let voxel_map = VoxelMap::new(map, bader_maxima);
        let densities =
            vec![vec![1., 1., 1., 2., 2., 1., 2., 2., 2., 1., 1., 1.],
                 vec![1., -1., 1., 0., 0., 1., 0., 0., 0., 1., -1., 1.],];
        assert_eq!(voxel_map.charge_sum(&densities),
                   (vec![vec![4., 10.], vec![2., 0.]], vec![4, 5], 3., 3))
    }

    #[test]
    fn voxel_map_surface_distance() {
        let mut map: Vec<isize> = vec![-1; 4usize.pow(3)];
        let mut data: Vec<f64> = vec![0.; 4usize.pow(3)];
        for i in [0usize, 1, 3, 4, 5, 7, 12, 13, 15, 16, 17, 19, 20, 21, 23,
                  28, 29, 31, 48, 49, 51, 52, 53, 55, 60, 61, 63].iter()
        {
            map[*i] = 0;
            data[*i] = 1.;
        }
        let bader_maxima = vec![-1, 0];
        let voxel_map = VoxelMap::new(map, bader_maxima);
        let lattice = Lattice::new([[3., 3., 0.], [-3., 3., 0.], [1., 1., 1.]]);
        let atoms = Atoms::new(lattice, vec![[0., 0., 0.]], String::new());
        let lattice = Lattice::new([[3., 3., 0.], [-3., 3., 0.], [1., 1., 1.]]);
        let density = Density::new(&data,
                                   [4, 4, 4],
                                   lattice.to_cartesian,
                                   Some(1E-3),
                                   [0., 0., 0.1]);
        assert_eq!(voxel_map.surface_distance(&vec![0], &atoms, &density),
                   vec![density.voxel_lattice
                               .distance_matrix
                               .iter()
                               .min_by(|a, b| a.partial_cmp(b).unwrap())
                               .unwrap()
                               .clone()
                        - 0.1
                          * utils::norm(density.voxel_lattice.to_cartesian[2])])
    }
}