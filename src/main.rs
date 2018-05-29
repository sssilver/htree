extern crate gif;

use gif::{Frame, Encoder, Repeat, SetParameter};
use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt;
use std::fs::File;

const IMGPX: usize = 36;

// -------------------------------------------------------------------------
// GEOMETRY

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
enum Dir {
    H, // Horizontal
    V, // Vertical
}

impl Dir {
    fn other(&self) -> Dir {
        return match self {
            Dir::H => Dir::V,
            Dir::V => Dir::H,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
struct Line {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl fmt::Debug for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{}).({},{})", self.x1, self.y1, self.x2, self.y2)
    }
}

impl Line {
    fn dir(self) -> Dir {
        if self.x1 == self.x2 {
            return Dir::V
        }
        Dir::H
    }

    fn len(self) -> i32 {
        match self.dir() {
            Dir::H => (self.x2 - self.x1).abs(),
            Dir::V => (self.y2 - self.y1).abs(),
        }
    }
}

// -------------------------------------------------------------------------
// HTREE FRACTALS

#[derive(Debug)]
struct HTree {
    t: i32,
    older: HashSet<Line>,
    newer: HashSet<Line>,
}

impl HTree {
    fn new() -> HTree {
        let mut start: HashSet<Line> = HashSet::new();
        start.insert(Line {
            x1: 200,
            y1: 200,
            x2: 200,
            y2: 300,
        });

        HTree {
            t: 0,
            older: HashSet::new(),
            newer: start,
        }
    }

    // Add one more level to the fractal. 
    fn tick(&self) -> HTree {
        HTree {
            t: self.t + 1,
            // Lines generated in the previous tick (`newer` lines) are now old.
            older: self.older.union(&self.newer).map(|x| x.to_owned()).collect(),
            // Make two new lines from each previous tick's lines.
            newer: self.newer.iter().flat_map(|l| HTree::two_new(*l)).collect(),
        }
    }

    // Use the H-Tree rules to generate two new lines from this one.
    // Each new line will be perpendicular to the current line and half its height.
    // The original line will bisect each of the two new lines.
    fn two_new(line: Line) -> Vec<Line> {

        fn line_from_center(x: i32, y: i32, dir: Dir, len: i32) -> Line {
            match dir {
                Dir::H => Line {
                    x1: x - len,
                    y1: y,
                    x2: x + len,
                    y2: y
                },
                Dir::V => Line {
                    x1: x,
                    y1: y - len,
                    x2: x,
                    y2: y + len,
                },
            }
        }

        vec! 
            [ line_from_center(line.x1, line.y1, line.dir().other(), line.len()/2)
            , line_from_center(line.x2, line.y2, line.dir().other(), line.len()/2)
            ]
    }
}

// -------------------------------------------------------------------------
// GIFS

fn write_image(
    filename: &str, 
    bitmaps: Vec<[u8; 36]>,
    bounds: (usize, usize)
) -> Result<(), HTreeError> {

    let mut file = File::create(filename)?;
    let color_map = 
        &[ 0xFF // Background R
         , 0xFF // Background G
         , 0xFF // Background B
         , 0xFF // Foreground R
         , 0xAA // Foreground G
         , 0    // Foreground B
         ] ;
    let mut encoder = Encoder::new(&mut file, bounds.0 as u16, bounds.1 as u16, color_map)?;
    encoder.set(Repeat::Infinite).unwrap();

    for bitmap in bitmaps {
        let mut frame = Frame::default();
        frame.width = bounds.0 as u16;
        frame.height = bounds.1 as u16;
        frame.buffer = Cow::Borrowed(&bitmap);
        encoder.write_frame(&frame).unwrap();
    }

    Ok(())
}

// -------------------------------------------------------------------------
// RUNNING

fn main() -> Result<(), HTreeError> {
    let size = 6;
    let bounds: (usize, usize) = (size, size);
    let filename = "htree.gif";

    // Test image drawing

    let p1 = [
        0, 0, 0, 0, 0, 0,
        0, 1, 1, 0, 0, 0,
        0, 1, 1, 0, 0, 0,
        0, 0, 0, 1, 1, 0,
        0, 0, 0, 1, 1, 0,
        0, 0, 0, 0, 0, 0,
    ];
    let p2 = [
        0, 0, 0, 0, 0, 0,
        0, 1, 1, 0, 0, 0,
        0, 1, 0, 0, 0, 0,
        0, 0, 0, 0, 1, 0,
        0, 0, 0, 1, 1, 0,
        0, 0, 0, 0, 0, 0,
    ];
    let bitmaps: Vec<[u8; IMGPX]> = vec![p1, p2];
    let _ = write_image(filename, bitmaps, bounds)?;

    // Test fractal

    let h0 = HTree::new();
    println!("{:?}", h0);
    let h1 = h0.tick();
    println!("{:?}", h1);
    let h2 = h1.tick();
    println!("{:?}", h2);

    Ok(())
}

#[derive(Debug)]
enum HTreeError {
    IOError(std::io::Error),
}

impl From<std::io::Error> for HTreeError {
    fn from(err: std::io::Error) -> Self {
        HTreeError::IOError(err)
    }
}