use crate::level::{Level, ObstacleKind, Tile, LEVEL_HEIGHT, LEVEL_WIDTH, TILE_SIZE};

use godot::prelude::*;
use num_integer::Roots;
use num_rational::Rational32;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn from_vector(vector: Vector2) -> Self {
        Self {
            x: (vector.x / TILE_SIZE) as usize,
            y: (vector.y / TILE_SIZE) as usize,
        }
    }

    pub fn to_vector(&self) -> Vector2 {
        Vector2::new(self.x as f32, self.y as f32) * TILE_SIZE
    }

    pub fn adjacent(&self) -> Vec<Self> {
        let mut positions = Vec::new();

        if self.x > 0 {
            positions.push(Position {
                x: self.x - 1,
                y: self.y,
            });
        }

        if self.x < LEVEL_WIDTH - 1 {
            positions.push(Position {
                x: self.x + 1,
                y: self.y,
            });
        }

        if self.y > 0 {
            positions.push(Position {
                x: self.x,
                y: self.y - 1,
            });
        }

        if self.y < LEVEL_HEIGHT - 1 {
            positions.push(Position {
                x: self.x,
                y: self.y + 1,
            });
        }

        positions
    }

    pub fn distance(&self, other: Self) -> u16 {
        let dx = self.x as i16 - other.x as i16;
        let dy = self.y as i16 - other.y as i16;
        (dx * dx + dy * dy).sqrt() as u16
    }

    pub fn direction_to(&self, other: Self) -> Direction {
        if other.x < self.x {
            Direction::Left
        } else if other.x > self.x {
            Direction::Right
        } else if other.y < self.y {
            Direction::Up
        } else if other.y > self.y {
            Direction::Down
        } else {
            unreachable!()
        }
    }

    pub fn in_direction(&self, direction: Direction, dist: usize) -> Option<Self> {
        match direction {
            Direction::Left => {
                if self.x < dist {
                    None
                } else {
                    Some(Position {
                        x: self.x - dist,
                        y: self.y,
                    })
                }
            }
            Direction::Right => {
                if self.x + dist >= LEVEL_WIDTH {
                    None
                } else {
                    Some(Position {
                        x: self.x + dist,
                        y: self.y,
                    })
                }
            }
            Direction::Up => {
                if self.y < dist {
                    None
                } else {
                    Some(Position {
                        x: self.x,
                        y: self.y - dist,
                    })
                }
            }
            Direction::Down => {
                if self.y + dist >= LEVEL_HEIGHT {
                    None
                } else {
                    Some(Position {
                        x: self.x,
                        y: self.y + dist,
                    })
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Frontier {
    priority: u16,
    position: Position,
}

impl Ord for Frontier {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for Frontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn pathfind(
    start: Position,
    goal: Position,
    grid: [[Tile; LEVEL_HEIGHT]; LEVEL_WIDTH],
    start_tile: Tile,
    dimensions: (usize, usize),
) -> Option<Vec<Position>> {
    let (width, height) = dimensions;
    let mut frontier = BinaryHeap::new();
    let mut came_from = HashMap::new();
    let mut costs = HashMap::new();

    frontier.push(Frontier {
        priority: 0,
        position: start,
    });
    costs.insert(start, 0);

    while let Some(Frontier {
        priority: _,
        position,
    }) = frontier.pop()
    {
        if position == goal {
            break;
        }

        'a: for adjacent in &position.adjacent() {
            for i in 0..width {
                for j in 0..height {
                    if adjacent.x + i >= LEVEL_WIDTH || adjacent.y + j >= LEVEL_HEIGHT {
                        continue 'a;
                    }

                    if grid[adjacent.x + i][adjacent.y + j] != start_tile {
                        if !grid[adjacent.x + i][adjacent.y + j].is_empty() {
                            continue 'a;
                        }
                    }
                }
            }

            let new_cost = costs.get(&position).unwrap() + 1;
            if !costs.contains_key(&adjacent) || new_cost < *costs.get(&adjacent).unwrap() {
                let diagonal = if position.x != adjacent.x && position.y != adjacent.y {
                    1
                } else {
                    0
                };
                frontier.push(Frontier {
                    priority: new_cost + adjacent.distance(goal) + diagonal,
                    position: *adjacent,
                });
                came_from.insert(*adjacent, Some(position));
                costs.insert(*adjacent, new_cost);
            }
        }
    }

    let mut position = goal;
    let mut path = Vec::new();

    while position != start {
        path.push(position);
        position = match came_from.get(&position) {
            Some(Some(position)) => *position,
            _ => return None,
        };
    }
    path.reverse();

    Some(path)
}

pub fn line_to(
    start: Position,
    goal: Position,
    grid: [[Tile; LEVEL_HEIGHT]; LEVEL_WIDTH],
) -> Option<Vec<Position>> {
    let distance = start.distance(goal) as usize;
    for direction in Direction::iter() {
        let mut path = Vec::new();
        for dist in 1..=distance {
            let position = match start.in_direction(direction, dist) {
                Some(position) => position,
                None => break,
            };

            if position == goal {
                path.push(position);
                return Some(path);
            }

            if grid[position.x][position.y].is_empty() {
                path.push(position);
            } else {
                break;
            }
        }
    }
    None
}

pub fn attack_positions(
    position: Position,
    range: u16,
    grid: [[Tile; LEVEL_HEIGHT]; LEVEL_WIDTH],
    dimensions: (usize, usize),
) -> Vec<(Position, u16)> {
    let (width, height) = dimensions;
    let mut positions = Vec::new();
    for i in 0..width {
        for j in 0..height {
            if position.x + i >= LEVEL_WIDTH || position.y + j >= LEVEL_HEIGHT {
                continue;
            }
            let position = Position {
                x: position.x + i,
                y: position.y + j,
            };
            for direction in Direction::iter() {
                for dist in 1..=range {
                    let position = match position.in_direction(direction, dist as usize) {
                        Some(position) => position,
                        None => break,
                    };

                    if grid[position.x][position.y].is_empty() {
                        positions.push((position, dist));
                    } else {
                        break;
                    }
                }
            }
        }
    }
    positions
}

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Cardinal {
    North,
    East,
    South,
    West,
}

#[derive(Debug, Clone, Copy)]
pub struct Quadrant {
    pub origin: Position,
    pub cardinal: Cardinal,
}

impl Quadrant {
    pub fn new(origin: Position, cardinal: Cardinal) -> Self {
        Self { origin, cardinal }
    }

    pub fn transform(&self, tile: (i32, i32)) -> Position {
        let (row, col) = tile;
        match self.cardinal {
            Cardinal::North => Position {
                x: (self.origin.x as i32 + col) as usize,
                y: (self.origin.y as i32 - row) as usize,
            },
            Cardinal::East => Position {
                x: (self.origin.x as i32 + col) as usize,
                y: (self.origin.y as i32 + row) as usize,
            },
            Cardinal::South => Position {
                x: (self.origin.x as i32 + row) as usize,
                y: (self.origin.y as i32 + col) as usize,
            },
            Cardinal::West => Position {
                x: (self.origin.x as i32 - row) as usize,
                y: (self.origin.y as i32 + col) as usize,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Row {
    pub depth: i32,
    pub start_slope: Rational32,
    pub end_slope: Rational32,
}

impl Row {
    pub fn new(depth: i32, start_slope: Rational32, end_slope: Rational32) -> Self {
        Self {
            depth,
            start_slope,
            end_slope,
        }
    }

    pub fn tiles(&self) -> Vec<(i32, i32)> {
        let depth = Rational32::from_integer(self.depth);
        let min_col = round_ties_up(depth * self.start_slope);
        let max_col = round_ties_down(depth * self.end_slope);
        (min_col..=max_col).map(|col| (self.depth, col)).collect()
    }

    pub fn next(&self) -> Self {
        Self::new(self.depth + 1, self.start_slope, self.end_slope)
    }
}

pub fn compute_fov(origin: Position, distance: u16, level: &Level) -> HashSet<Position> {
    let mut visible = HashSet::new();
    visible.insert(origin);
    for cardinal in Cardinal::iter() {
        let quadrant = Quadrant::new(origin, cardinal);
        let first_row = Row::new(1, Rational32::from_integer(-1), Rational32::from_integer(1));
        visible.extend(scan(quadrant, first_row, distance, level));
    }

    visible
}

fn scan(quadrant: Quadrant, mut row: Row, distance: u16, level: &Level) -> HashSet<Position> {
    if distance == 0 {
        return HashSet::new();
    }

    let mut prev_position = None;
    let mut visible = HashSet::new();

    for tile in row.tiles() {
        let position = quadrant.transform(tile);

        if is_wall(position, level) || is_symmetric(row, tile) {
            visible.insert(position);
        }

        match prev_position {
            Some(prev_position) => {
                if is_wall(prev_position, level) && !is_wall(position, level) {
                    row.start_slope = slope(tile);
                }

                if !is_wall(prev_position, level) && is_wall(position, level) {
                    let mut next_row = row.next();
                    next_row.end_slope = slope(tile);
                    visible.extend(scan(quadrant, next_row, distance - 1, level));
                }
            }
            None => (),
        }

        prev_position = Some(position);
    }

    match prev_position {
        Some(prev_position) if !is_wall(prev_position, level) => {
            visible.extend(scan(quadrant, row.next(), distance - 1, level));
        }
        _ => (),
    }

    visible
}

fn is_wall(position: Position, level: &Level) -> bool {
    if position.x >= LEVEL_WIDTH || position.y >= LEVEL_HEIGHT {
        true
    } else {
        match level.grid[position.x][position.y] {
            Tile::Obstacle(id) => {
                let obstacle = level.get_obstacle(id);
                let obstacle = obstacle.bind();
                match obstacle.kind {
                    ObstacleKind::Wall | ObstacleKind::Barrel => true,
                    ObstacleKind::LowWall => false,
                }
            }
            _ => false,
        }
    }
}

fn slope(tile: (i32, i32)) -> Rational32 {
    let (row, col) = tile;
    Rational32::new(2 * col - 1, 2 * row)
}

fn is_symmetric(row: Row, tile: (i32, i32)) -> bool {
    let (_, col) = tile;
    let col = Rational32::from_integer(col);
    let depth = Rational32::from_integer(row.depth);
    col >= depth * row.start_slope && col <= depth * row.end_slope
}

fn round_ties_up(n: Rational32) -> i32 {
    (n + Rational32::new(1, 2)).floor().to_integer()
}

fn round_ties_down(n: Rational32) -> i32 {
    (n - Rational32::new(1, 2)).ceil().to_integer()
}
