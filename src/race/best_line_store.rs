//! The best line worked out at a stage's load: its camera sets, map places and distances.

/// One set of replay cameras.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct CameraSet {
    /// The camera faces, at most 0x400.
    pub faces: Vec<u32>,
    /// Each checkpoint's nearest camera (empty without cameras).
    pub nearest: Vec<u32>,
}

/// What the work leaves in the variables others read (None: never written).
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Leftovers {
    /// The scene's scratch word.
    pub scratch: Option<u32>,
}

/// A checkpoint's place on the map.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MapPlace {
    pub checkpoint: u32,
    pub x: u32,
    pub y: u32,
}

/// What a line of two checkpoints or more has besides its chain.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Along {
    /// The (place, angle) writes of the angles table, in write order.
    pub angles: Vec<(u32, u32)>,
    /// Each checkpoint's count of checks.
    pub check_counts: Vec<u32>,
    /// The distance along the line at each checkpoint; the last is the line's length.
    pub distances: Vec<u32>,
    /// How many map anchors the line has.
    pub map_anchors: u32,
    /// Each mapped checkpoint's place on the map.
    pub map: Vec<MapPlace>,
    /// The two replay camera sets.
    pub cameras: [CameraSet; 2],
    /// The share of the line on each of the 16 surfaces, whole numbers.
    pub surface_shares: [u32; 16],
}

/// The best line of the loaded course.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BestLine {
    /// The face kind of the line's faces for the stage's direction.
    pub line_face_kind: u32,
    /// The faces of that kind, the chain first (all of them in the order the faces come when
    /// there are fewer than two).
    pub faces: Vec<u32>,
    /// The chain's length (0 with fewer than two faces): the line's face total and the
    /// checkpoint count.
    pub checkpoint_count: u32,
    /// The last checkpoint's number: the count less 1 (None, and the table keeps its value, with
    /// fewer than two line faces).
    pub last_checkpoint: Option<u32>,
    /// The rest, when there are two checkpoints or more.
    pub along: Option<Along>,
    /// What the work leaves in the variables.
    pub leftovers: Leftovers,
}
