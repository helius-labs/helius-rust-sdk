use serde::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AssetSortBy {
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "updated")]
    Updated,
    #[serde(rename = "recent_action")]
    RecentAction,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AssetSortDirection {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}