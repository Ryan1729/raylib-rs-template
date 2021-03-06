#![deny(unused)]
#![deny(bindings_with_variant_name)]

#[allow(unused)]
macro_rules! compile_time_assert {
    ($assertion: expr) => {{
        #[allow(unknown_lints, eq_op)]
        // Based on the const_assert macro from static_assertions;
        const _: [(); 0 - !{$assertion} as usize] = [];
    }}
}

// In case we decide that we care about no_std/not directly allocating ourself
pub trait ClearableStorage<A> {
    fn clear(&mut self);

    fn push(&mut self, a: A);
}

pub type Seed = [u8; 16];

type Xs = [core::num::Wrapping<u32>; 4];

fn xorshift(xs: &mut Xs) -> u32 {
    let mut t = xs[3];

    xs[3] = xs[2];
    xs[2] = xs[1];
    xs[1] = xs[0];

    t ^= t << 11;
    t ^= t >> 8;
    xs[0] = t ^ xs[0] ^ (xs[0] >> 19);

    xs[0].0
}

#[allow(unused)]
fn xs_u32(xs: &mut Xs, min: u32, one_past_max: u32) -> u32 {
    (xorshift(xs) % (one_past_max - min)) + min
}

#[allow(unused)]
fn xs_shuffle<A>(rng: &mut Xs, slice: &mut [A]) {
    for i in 1..slice.len() as u32 {
        // This only shuffles the first u32::MAX_VALUE - 1 elements.
        let r = xs_u32(rng, 0, i + 1) as usize;
        let i = i as usize;
        slice.swap(i, r);
    }
}

#[allow(unused)]
fn new_seed(rng: &mut Xs) -> Seed {
    let s0 = xorshift(rng).to_le_bytes();
    let s1 = xorshift(rng).to_le_bytes();
    let s2 = xorshift(rng).to_le_bytes();
    let s3 = xorshift(rng).to_le_bytes();

    [
        s0[0], s0[1], s0[2], s0[3],
        s1[0], s1[1], s1[2], s1[3],
        s2[0], s2[1], s2[2], s2[3],
        s3[0], s3[1], s3[2], s3[3],
    ]
}

fn xs_from_seed(mut seed: Seed) -> Xs {
    // 0 doesn't work as a seed, so use this one instead.
    if seed == [0; 16] {
        seed = 0xBAD_5EED_u128.to_le_bytes();
    }

    macro_rules! wrap {
        ($i0: literal, $i1: literal, $i2: literal, $i3: literal) => {
            core::num::Wrapping(
                u32::from_le_bytes([
                    seed[$i0],
                    seed[$i1],
                    seed[$i2],
                    seed[$i3],
                ])
            )
        }
    }

    [
        wrap!( 0,  1,  2,  3),
        wrap!( 4,  5,  6,  7),
        wrap!( 8,  9, 10, 11),
        wrap!(12, 13, 14, 15),
    ]
}

/// This type alias makes adding a custom newtype easy.
pub type X = f32;
/// This type alias makes adding a custom newtype easy.
pub type Y = f32;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct XY {
    pub x: X,
    pub y: Y,
}

pub mod draw;

pub use draw::{
    DrawLength,
    DrawX,
    DrawY,
    DrawXY,
    DrawW,
    DrawH,
    DrawWH,
    SpriteKind,
    SpriteSpec,
    Sizes,
};

macro_rules! from_rng_enum_def {
    ($name: ident { $( $variants: ident ),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $name {
            $( $variants ),+
        }

        impl $name {
            pub const COUNT: usize = {
                let mut count = 0;

                $(
                    // Some reference to the vars is needed to use
                    // the repetitions.
                    let _ = Self::$variants;

                    count += 1;
                )+

                count
            };

            pub const ALL: [Self; Self::COUNT] = [
                $(Self::$variants,)+
            ];

            pub fn from_rng(rng: &mut Xs) -> Self {
                Self::ALL[xs_u32(rng, 0, Self::ALL.len() as u32) as usize]
            }
        }
    }
}

from_rng_enum_def!{
    ArrowKind {
        Red,
        Green
    }
}

impl Default for ArrowKind {
    fn default() -> Self {
        Self::Red
    }
}

from_rng_enum_def!{
    Dir {
        Up,
        UpRight,
        Right,
        DownRight,
        Down,
        DownLeft,
        Left,
        UpLeft,
    }
}

impl Default for Dir {
    fn default() -> Self {
        Self::Up
    }
}

mod tile {
    use crate::{Xs, xs_u32};

    pub type Count = u32;

    pub type Coord = u8;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct X(Coord);

    impl X {
        pub const MAX: Coord = 0b1111;
        pub const COUNT: Count = (X::MAX as Count) + 1;

        pub fn from_rng(rng: &mut Xs) -> Self {
            Self(xs_u32(rng, 0, Self::COUNT) as Coord)
        }

        pub fn saturating_add_one(&self) -> Self {
            Self(core::cmp::min(self.0.saturating_add(1), Self::MAX))
        }

        pub fn saturating_sub_one(&self) -> Self {
            Self(self.0.saturating_sub(1))
        }
    }

    impl From<X> for Coord {
        fn from(X(c): X) -> Self {
            c
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Y(Coord);

    impl Y {
        pub const MAX: Coord = 0b1111;
        pub const COUNT: Count = (Y::MAX as Count) + 1;

        pub fn from_rng(rng: &mut Xs) -> Self {
            Self(xs_u32(rng, 0, Self::COUNT) as Coord)
        }

        pub fn saturating_add_one(&self) -> Self {
            Self(core::cmp::min(self.0.saturating_add(1), Self::MAX))
        }

        pub fn saturating_sub_one(&self) -> Self {
            Self(self.0.saturating_sub(1))
        }
    }

    impl From<Y> for Coord {
        fn from(Y(c): Y) -> Self {
            c
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct XY {
        pub x: X,
        pub y: Y,
    }

    impl XY {
        pub const COUNT: Count = X::COUNT * Y::COUNT;

        pub fn from_rng(rng: &mut Xs) -> Self {
            Self {
                x: X::from_rng(rng),
                y: Y::from_rng(rng),
            }
        }

        pub fn move_up(&mut self) {
            self.y = self.y.saturating_sub_one();
        }

        pub fn move_down(&mut self) {
            self.y = self.y.saturating_add_one();
        }

        pub fn move_left(&mut self) {
            self.x = self.x.saturating_sub_one();
        }

        pub fn move_right(&mut self) {
            self.x = self.x.saturating_add_one();
        }
    }

    #[allow(unused)]
    pub fn xy_to_i(xy: XY) -> usize {
        xy_to_i_usize((usize::from(xy.x.0), usize::from(xy.y.0)))
    }

    pub fn xy_to_i_usize((x, y): (usize, usize)) -> usize {
        y * Y::COUNT as usize + x
    }

    pub fn i_to_xy(index: usize) -> XY {
        XY {
            x: X(to_coord_or_default(
                (index % X::COUNT as usize) as Count
            )),
            y: Y(to_coord_or_default(
                ((index % (XY::COUNT as usize) as usize)
                / X::COUNT as usize) as Count
            )),
        }
    }

    fn to_coord_or_default(n: Count) -> Coord {
        core::convert::TryFrom::try_from(n).unwrap_or_default()
    }
}

fn draw_xy_from_tile(sizes: &Sizes, txy: tile::XY) -> DrawXY {
    DrawXY {
        x: sizes.board_xywh.x + sizes.board_xywh.w * (tile::Coord::from(txy.x) as DrawLength / tile::X::COUNT as DrawLength),
        y: sizes.board_xywh.y + sizes.board_xywh.h * (tile::Coord::from(txy.y) as DrawLength / tile::Y::COUNT as DrawLength),
    }
}

fn draw_xy_from_offset(sizes: &Sizes, o_xy: offset::XY) -> DrawXY {
    let offset_size = sizes.tile_side_length / TILE_OFFSET as DrawLength;

    DrawXY {
        x: o_xy.x.0 as DrawLength * offset_size,
        y: o_xy.y.0 as DrawLength * offset_size,
    }
}

/// A Tile should always be at a particular position, but that position should be
/// derivable from the tiles location in the tiles array, so it doesn't need to be
/// stored. But, we often want to get the tile's data and it's location as a single
/// thing. This is why we have both `Tile` and `TileData`
#[derive(Copy, Clone, Debug, Default)]
struct TileData {
    dir: Dir,
    arrow_kind: ArrowKind,
}

impl TileData {
    fn from_rng(rng: &mut Xs) -> Self {
        Self {
            dir: Dir::from_rng(rng),
            arrow_kind: ArrowKind::from_rng(rng),
        }
    }

    fn sprite(&self) -> SpriteKind {
        SpriteKind::Arrow(self.dir, self.arrow_kind)
    }
}

pub const TILES_LENGTH: usize = tile::XY::COUNT as _;

type TileDataArray = [TileData; TILES_LENGTH as _];

#[derive(Clone, Debug)]
pub struct Tiles {
    tiles: TileDataArray,
}

impl Default for Tiles {
    fn default() -> Self {
        Self {
            tiles: [TileData::default(); TILES_LENGTH as _],
        }
    }
}

impl Tiles {
    fn from_rng(rng: &mut Xs) -> Self {
        let mut tiles = [TileData::default(); TILES_LENGTH as _];

        for tile_data in tiles.iter_mut() {
            *tile_data = TileData::from_rng(rng);
        }

        Self {
            tiles
        }
    }
}

#[derive(Debug)]
enum EyeState {
    Idle,
    Moved(Dir),
    NarrowAnimLeft,
    NarrowAnimCenter,
    NarrowAnimRight,
    SmallPupil,
    Closed,
    HalfLid,
}

impl Default for EyeState {
    fn default() -> Self {
        Self::Idle
    }
}

impl EyeState {
    fn sprite(&self) -> SpriteKind {
        use EyeState::*;
        match self {
            Idle => SpriteKind::NeutralEye,
            Moved(dir) => SpriteKind::DirEye(*dir),
            NarrowAnimLeft => SpriteKind::NarrowLeftEye,
            NarrowAnimCenter => SpriteKind::NarrowCenterEye,
            NarrowAnimRight => SpriteKind::NarrowRightEye,
            SmallPupil => SpriteKind::SmallPupilEye,
            Closed => SpriteKind::ClosedEye,
            HalfLid => SpriteKind::HalfLidEye,
        }
    }
}

mod offset {
    use core::ops::{AddAssign, Add, SubAssign, Sub};

    pub type Offset = i8;

    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct X(pub Offset);

    impl AddAssign for X {
        fn add_assign(&mut self, other: Self) {
            self.0 += other.0;
        }
    }

    impl Add for X {
        type Output = Self;

        fn add(mut self, other: Self) -> Self::Output {
            self += other;
            self
        }
    }

    impl SubAssign for X {
        fn sub_assign(&mut self, other: Self) {
            self.0 -= other.0;
        }
    }

    impl Sub for X {
        type Output = Self;

        fn sub(mut self, other: Self) -> Self::Output {
            self -= other;
            self
        }
    }

    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Y(pub Offset);

    impl AddAssign for Y {
        fn add_assign(&mut self, other: Self) {
            self.0 += other.0;
        }
    }

    impl Add for Y {
        type Output = Self;

        fn add(mut self, other: Self) -> Self::Output {
            self += other;
            self
        }
    }

    impl SubAssign for Y {
        fn sub_assign(&mut self, other: Self) {
            self.0 -= other.0;
        }
    }

    impl Sub for Y {
        type Output = Self;

        fn sub(mut self, other: Self) -> Self::Output {
            self -= other;
            self
        }
    }

    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct XY {
        pub x: X,
        pub y: Y,
    }

    impl AddAssign for XY {
        fn add_assign(&mut self, other: Self) {
            self.x += other.x;
            self.y += other.y;
        }
    }

    impl Add for XY {
        type Output = Self;

        fn add(mut self, other: Self) -> Self::Output {
            self += other;
            self
        }
    }

    impl SubAssign for XY {
        fn sub_assign(&mut self, other: Self) {
            self.x -= other.x;
            self.y -= other.y;
        }
    }

    impl Sub for XY {
        type Output = Self;

        fn sub(mut self, other: Self) -> Self::Output {
            self -= other;
            self
        }
    }

    #[macro_export]
    macro_rules! o_xy {
        () => {
            o_xy!{0, 0}
        };
        ($x: expr, $y: expr $(,)?) => {
            offset::XY {
                x: offset::X(
                    $x,
                ),
                y: offset::Y(
                    $y,
                ),
            }
        }
    }
}

const TILE_OFFSET: offset::Offset = 16;

#[derive(Debug, Default)]
struct Eye {
    xy: tile::XY,
    offset_xy: offset::XY,
    state: EyeState,
}

#[derive(Debug, Default)]
struct Board {
    tiles: Tiles,
    eye: Eye,
}

impl Board {
    fn from_seed(seed: Seed) -> Self {
        let mut rng = xs_from_seed(seed);

        let tiles = Tiles::from_rng(&mut rng);

        Self {
            tiles,
            eye: Eye {
                xy: tile::XY::from_rng(&mut rng),
                ..<_>::default()
            },
        }
    }
}

/// 64k animation frames ought to be enough for anybody!
type AnimationTimer = u16;

/// We use this because it has a lot more varied factors than 65536.
const ANIMATION_TIMER_LENGTH: AnimationTimer = 60 * 60 * 18;

#[derive(Debug, Default)]
pub struct State {
    sizes: draw::Sizes,
    board: Board,
    animation_timer: AnimationTimer
}

impl State {
    pub fn from_seed(seed: Seed) -> Self {
        Self {
            board: Board::from_seed(seed),
            ..<_>::default()
        }
    }
}

pub fn sizes(state: &State) -> draw::Sizes {
    state.sizes.clone()
}

pub type InputFlags = u16;

pub const INPUT_UP_PRESSED: InputFlags              = 0b0000_0000_0000_0001;
pub const INPUT_DOWN_PRESSED: InputFlags            = 0b0000_0000_0000_0010;
pub const INPUT_LEFT_PRESSED: InputFlags            = 0b0000_0000_0000_0100;
pub const INPUT_RIGHT_PRESSED: InputFlags           = 0b0000_0000_0000_1000;

pub const INPUT_UP_DOWN: InputFlags                 = 0b0000_0000_0001_0000;
pub const INPUT_DOWN_DOWN: InputFlags               = 0b0000_0000_0010_0000;
pub const INPUT_LEFT_DOWN: InputFlags               = 0b0000_0000_0100_0000;
pub const INPUT_RIGHT_DOWN: InputFlags              = 0b0000_0000_1000_0000;

pub const INPUT_INTERACT_PRESSED: InputFlags        = 0b0000_0001_0000_0000;

#[derive(Clone, Copy, Debug)]
enum Input {
    NoChange,
    Dir(Dir),
    Interact,
}

impl Input {
    fn from_flags(flags: InputFlags) -> Self {
        use Input::*;
        use crate::Dir::*;
        if INPUT_INTERACT_PRESSED & flags != 0 {
            Interact
        } else if (INPUT_UP_DOWN | INPUT_RIGHT_DOWN) & flags == (INPUT_UP_DOWN | INPUT_RIGHT_DOWN) {
            Dir(UpRight)
        } else if (INPUT_DOWN_DOWN | INPUT_RIGHT_DOWN) & flags == (INPUT_DOWN_DOWN | INPUT_RIGHT_DOWN) {
            Dir(DownRight)
        } else if (INPUT_DOWN_DOWN | INPUT_LEFT_DOWN) & flags == (INPUT_DOWN_DOWN | INPUT_LEFT_DOWN) {
            Dir(DownLeft)
        } else if (INPUT_UP_DOWN | INPUT_LEFT_DOWN) & flags == (INPUT_UP_DOWN | INPUT_LEFT_DOWN) {
            Dir(UpLeft)
        } else if INPUT_UP_DOWN & flags != 0 {
            Dir(Up)
        } else if INPUT_DOWN_DOWN & flags != 0 {
            Dir(Down)
        } else if INPUT_LEFT_DOWN & flags != 0 {
            Dir(Left)
        } else if INPUT_RIGHT_DOWN & flags != 0 {
            Dir(Right)
        } else {
            NoChange
        }
    }
}

fn update_step(
    state: &mut State,
    input: Input,
) {
    use EyeState::*;
    use Input::*;
    use crate::Dir::*;

    const HOLD_FRAMES: AnimationTimer = 30;

    if state.board.eye.offset_xy == o_xy!{} {
        macro_rules! offset_if_moved {
            ($($tile_method: ident)+ {$($offset_tokens: tt)*}) => {
                let old_xy = state.board.eye.xy;
                $(
                    state.board.eye.xy.$tile_method();
                )+
                let o_xy = o_xy!{$($offset_tokens)*};
                if state.board.eye.xy.x != old_xy.x {
                    state.board.eye.offset_xy.x = o_xy.x;
                }
                if state.board.eye.xy.y != old_xy.y {
                    state.board.eye.offset_xy.y = o_xy.y;
                }
            }
        }

        match input {
            NoChange => match state.board.eye.state {
                Idle => {
                    if state.animation_timer % (HOLD_FRAMES * 3) == 0 {
                        state.board.eye.state = NarrowAnimCenter;
                    }
                },
                Moved(_) => {
                    if state.animation_timer % HOLD_FRAMES == 0 {
                        state.board.eye.state = Idle;
                    }
                },
                SmallPupil => {
                    if state.animation_timer % (HOLD_FRAMES * 3) == 0 {
                        state.board.eye.state = Closed;
                    }
                },
                Closed => {
                    if state.animation_timer % (HOLD_FRAMES) == 0 {
                        state.board.eye.state = HalfLid;
                    }
                },
                HalfLid => {
                    if state.animation_timer % (HOLD_FRAMES * 5) == 0 {
                        state.board.eye.state = Idle;
                    }
                },
                NarrowAnimCenter => {
                    let modulus = state.animation_timer % (HOLD_FRAMES * 4);
                    if modulus == 0 {
                        state.board.eye.state = NarrowAnimRight;
                    } else if modulus == HOLD_FRAMES * 2 {
                        state.board.eye.state = NarrowAnimLeft;
                    }
                },
                NarrowAnimLeft | NarrowAnimRight => {
                    if state.animation_timer % HOLD_FRAMES == 0 {
                        state.board.eye.state = NarrowAnimCenter;
                    }
                },
            },
            Dir(Up) => {
                state.board.eye.state = Moved(Up);
                offset_if_moved!{
                    move_up
                    {
                        0,
                        TILE_OFFSET
                    }
                }
            },
            Dir(UpRight) => {
                state.board.eye.state = Moved(UpRight);
                offset_if_moved!{
                    move_up
                    move_right
                    {
                        -TILE_OFFSET,
                        TILE_OFFSET
                    }
                }
            },
            Dir(Right) => {
                state.board.eye.state = Moved(Right);
                offset_if_moved!{
                    move_right
                    {
                        -TILE_OFFSET,
                        0,
                    }
                }
            },
            Dir(DownRight) => {
                state.board.eye.state = Moved(DownRight);
                offset_if_moved!{
                    move_down
                    move_right
                    {
                        -TILE_OFFSET,
                        -TILE_OFFSET,
                    }
                }
            },
            Dir(Down) => {
                state.board.eye.state = Moved(Down);
                offset_if_moved!{
                    move_down
                    {
                        0,
                        -TILE_OFFSET,
                    }
                }
            },
            Dir(DownLeft) => {
                state.board.eye.state = Moved(DownLeft);
                offset_if_moved!{
                    move_down
                    move_left
                    {
                        TILE_OFFSET,
                        -TILE_OFFSET,
                    }
                }
            },
            Dir(Left) => {
                state.board.eye.state = Moved(Left);
                offset_if_moved!{
                    move_left
                    {
                        TILE_OFFSET,
                        0,
                    }
                }
            },
            Dir(UpLeft) => {
                state.board.eye.state = Moved(UpLeft);
                offset_if_moved!{
                    move_up
                    move_left
                    {
                        TILE_OFFSET,
                        TILE_OFFSET,
                    }
                }
            },
            Interact => {
                state.board.eye.state = SmallPupil;
            },
        }
    } else {
        let o_xy = &mut state.board.eye.offset_xy;

        if o_xy.x > offset::X(0) {
           o_xy.x -= offset::X(1);
        } else if o_xy.x < offset::X(0) {
           o_xy.x += offset::X(1);
        } else {
            // Leave at zero
        }

        if o_xy.y > offset::Y(0) {
           o_xy.y -= offset::Y(1);
        } else if o_xy.y < offset::Y(0) {
           o_xy.y += offset::Y(1);
        } else {
            // Leave at zero
        }
    }

    state.animation_timer += 1;
    if state.animation_timer >= ANIMATION_TIMER_LENGTH {
        state.animation_timer = 0;
    }
}

pub type DeltaTimeInSeconds = f32;

const S_PER_UPDATE: DeltaTimeInSeconds = 1./240.;

pub fn update(
    state: &mut State,
    commands: &mut dyn ClearableStorage<draw::Command>,
    input_flags: InputFlags,
    draw_wh: DrawWH,
    mut dt: DeltaTimeInSeconds,
) {
    use draw::{TextSpec, TextKind, Command::*};

    if draw_wh != state.sizes.draw_wh {
        state.sizes = draw::fresh_sizes(draw_wh);
    }

    commands.clear();

    let input = Input::from_flags(input_flags);

    while dt >= S_PER_UPDATE {
        update_step(state, input);

        dt -= S_PER_UPDATE;
    }

    for i in 0..TILES_LENGTH {
        let tile_data = state.board.tiles.tiles[i];

        let txy = tile::i_to_xy(i);

        commands.push(Sprite(SpriteSpec{
            sprite: tile_data.sprite(),
            xy: draw_xy_from_tile(&state.sizes, txy),
        }));
    }

    commands.push(Sprite(SpriteSpec{
        sprite: state.board.eye.state.sprite(),
        xy: draw_xy_from_tile(&state.sizes, state.board.eye.xy)
        + draw_xy_from_offset(&state.sizes, state.board.eye.offset_xy),
    }));

    let left_text_x = state.sizes.play_xywh.x + MARGIN;

    const MARGIN: f32 = 16.;

    let small_section_h = state.sizes.draw_wh.h / 8. - MARGIN;

    {
        let mut y = MARGIN;

        commands.push(Text(TextSpec{
            text: format!(
                "input: {:?}",
                input
            ),
            xy: DrawXY { x: left_text_x, y },
            wh: DrawWH {
                w: state.sizes.play_xywh.w,
                h: small_section_h
            },
            kind: TextKind::UI,
        }));

        y += small_section_h;

        commands.push(Text(TextSpec{
            text: format!(
                "sizes: {:?}\nanimation_timer: {:?}",
                state.sizes,
                state.animation_timer
            ),
            xy: DrawXY { x: left_text_x, y },
            wh: DrawWH {
                w: state.sizes.play_xywh.w,
                h: state.sizes.play_xywh.h - y
            },
            kind: TextKind::UI,
        }));
    }
}
