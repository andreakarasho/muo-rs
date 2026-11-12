//! A starting city offered at character creation.

use muo_primitives::Point3D;

use crate::map::MapId;

/// `Map.Trammel`.
const TRAMMEL: MapId = MapId(1);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CityInfo {
    pub city: String,
    pub building: String,
    pub description: i32,
    pub location: Point3D,
    pub map: MapId,
}

impl CityInfo {
    /// `CityInfo(city, building, description, x, y, z, map)`.
    pub fn new(
        city: &str,
        building: &str,
        description: i32,
        x: i32,
        y: i32,
        z: i32,
        map: MapId,
    ) -> Self {
        Self {
            city: city.to_string(),
            building: building.to_string(),
            description,
            location: Point3D::new(x, y, z),
            map,
        }
    }

    /// `CityInfo(city, building, x, y, z, map)`.
    pub fn on_map(city: &str, building: &str, x: i32, y: i32, z: i32, map: MapId) -> Self {
        Self::new(city, building, 0, x, y, z, map)
    }

    /// `CityInfo(city, building, description, x, y, z)`: Trammel.
    pub fn described(city: &str, building: &str, description: i32, x: i32, y: i32, z: i32) -> Self {
        Self::new(city, building, description, x, y, z, TRAMMEL)
    }

    /// `CityInfo(city, building, x, y, z)`: Trammel.
    pub fn at(city: &str, building: &str, x: i32, y: i32, z: i32) -> Self {
        Self::new(city, building, 0, x, y, z, TRAMMEL)
    }

    pub fn x(&self) -> i32 {
        self.location.x
    }

    pub fn y(&self) -> i32 {
        self.location.y
    }

    pub fn z(&self) -> i32 {
        self.location.z
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_trammel() {
        let c = CityInfo::at("Britain", "Inn", 1, 2, 3);
        assert_eq!(
            (c.map, c.description, c.x(), c.y(), c.z()),
            (MapId(1), 0, 1, 2, 3)
        );
    }
}
