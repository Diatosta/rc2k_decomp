//! The menu carousel: the front end sets it up and turns it, the scene draws it and chooses its
//! pictures and textures. One struct of atomic cells, owned by `Screens`.

use crate::menu::lists_store::CarList;
use crate::session::PlainDword;

/// The menu carousel the front end keeps and the scene draws: the entry shown, the list, the
/// turn, the place of its entries, their kinds, and the menu's own numbers.
#[derive(rc2k_derive::Cells)]
pub struct Carousel {
    /// The kind of the entry being drawn.
    pub kind: PlainDword,
    /// The menu's list of cars, `CarList::code` (0: none); `Carousel::list`.
    pub entries: PlainDword,
    /// 1 after right, -1 after left, 0 at rest.
    pub direction: PlainDword,
    /// The x spacing of the places.
    pub spacing: PlainDword,
    /// The y of the places.
    pub y: PlainDword,
    /// The z of the places.
    pub z: PlainDword,
    /// The way forward (the front entry's step).
    pub forward: PlainDword,
    /// The entries' kinds (room for 0x100: the carousel checks that many).
    pub kinds: [PlainDword; 256],
    /// The menu whose carousel is up.
    pub menu: PlainDword,
    /// The entries.
    pub count: PlainDword,
    /// The entry in front.
    pub front: PlainDword,
    /// The turn's angle (0 after right, -1 after left).
    pub angle: PlainDword,
    /// The clock at the last `menu::step_carousel`.
    pub last_clock: PlainDword,
    /// The time since, held to 100.
    pub passed: PlainDword,
    /// Set to 1 by `scene::set_up_carousel` when the carousel's set-up is done; only the
    /// carousel set-up lists (`scene::carousel_set_ups`) name it.
    pub set_up: PlainDword,
    /// The carousel's pictures: four wanted, four shown, four held.
    pub pictures: CarouselPictures,
    /// The eight texture numbers of the carousel's menu (-1 none).
    pub textures: [PlainDword; 8],
}

impl Carousel {
    /// The list the carousel shows, none before one is set up.
    pub fn list(&self) -> Result<Option<CarList>, crate::errors::BadInput> {
        Ok(CarList::from_code(self.entries.get())?)
    }

    pub fn set_list(&self, list: CarList) {
        self.entries.set(list.code());
    }
}

/// The carousel's pictures: four wanted, four shown, four held.
#[derive(rc2k_derive::Cells)]
pub struct CarouselPictures {
    /// The pictures wanted (`scene::draw_car_object`).
    pub wanted: [PlainDword; 4],
    /// The pictures shown.
    pub shown: [PlainDword; 4],
    /// The pictures held (`menu::clear_choice_marks` sets them to -1).
    pub held: [PlainDword; 4],
}

impl Carousel {
    pub const fn new() -> Self {
        Carousel {
            kind: PlainDword::new(0),
            entries: PlainDword::new(0),
            direction: PlainDword::new(0),
            spacing: PlainDword::new(0),
            y: PlainDword::new(0),
            z: PlainDword::new(0),
            forward: PlainDword::new(0),
            kinds: [const { PlainDword::new(0) }; 256],
            menu: PlainDword::new(0),
            count: PlainDword::new(4),
            front: PlainDword::new(3),
            angle: PlainDword::new(0),
            last_clock: PlainDword::new(0),
            passed: PlainDword::new(0),
            set_up: PlainDword::new(1),
            pictures: CarouselPictures {
                wanted: [const { PlainDword::new(0) }; 4],
                shown: [const { PlainDword::new(0) }; 4],
                held: [const { PlainDword::new(0) }; 4],
            },
            textures: [const { PlainDword::new(0) }; 8],
        }
    }
}
