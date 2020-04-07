use frontmatter;
use yaml_rust::yaml::{Hash, Yaml};

pub struct Metadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

pub async fn read_metadata(source: &str) -> Metadata {
    let metadata = frontmatter::parse(&source)
        .unwrap_or(None)
        .and_then(|data| data.into_hash());

    if let Some(data) = metadata.as_ref() {
        let tags = read_tags(data);
        let title = read_str_field("title", data);
        let description = read_str_field("description", data);

        Metadata {
            tags,
            title,
            description,
        }
    } else {
        Metadata {
            title: None,
            description: None,
            tags: None,
        }
    }
}

pub fn read_str_field(key: &str, map: &Hash) -> Option<String> {
    map.get(&Yaml::String(key.to_string()))
        .and_then(|v| v.as_str())
        .map(|v| v.trim().to_string())
}

pub fn read_tags(map: &Hash) -> Option<Vec<String>> {
    get_tags(map).and_then(decode_tags)
}

fn get_tags(map: &Hash) -> Option<&Yaml> {
    map.get(&Yaml::String(format!("tags")))
        .or_else(|| map.get(&Yaml::String(format!("Tags"))))
}

fn decode_tags(value: &Yaml) -> Option<Vec<String>> {
    decode_tags_array(value).or_else(|| decode_tags_string(value))
}

fn decode_tags_array(value: &Yaml) -> Option<Vec<String>> {
    value.as_vec().as_ref().map(|vec| {
        vec.into_iter()
            .filter_map(decode_tag)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    })
}

fn decode_tags_string(value: &Yaml) -> Option<Vec<String>> {
    value.as_str().as_ref().map(|s| {
        s.rsplit(",")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    })
}

fn decode_tag(value: &Yaml) -> Option<&str> {
    if let Yaml::String(tag) = value {
        Some(tag)
    } else {
        None
    }
}
