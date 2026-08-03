use std::collections::HashMap;

pub fn extract_visuals(files: &mut HashMap<String, Vec<u8>>) -> HashMap<String, Vec<u8>> {
    files.extract_if(|path, _| path.starts_with("Images/Main") && path.to_lowercase().ends_with(".jpg")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_main_jpgs() {
        let mut files = HashMap::from([
            ("Images/Main/A.jpg".to_string(), vec![1u8]),
            ("Images/Main/B.JPG".to_string(), vec![2u8]),   // case-insensitivity
            ("Images/Main/IR.data".to_string(), vec![3u8]), // wrong extension
            ("Other/C.jpg".to_string(), vec![4u8]),         // wrong directory
        ]);
        let visuals = extract_visuals(&mut files);

        assert_eq!(visuals.len(), 2);
        assert!(visuals.contains_key("Images/Main/A.jpg"));
        assert!(visuals.contains_key("Images/Main/B.JPG"));
        assert_eq!(files.len(), 2); // extracted entries were moved out
    }
}
