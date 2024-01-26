pub fn split(command: String) -> Vec<String> {
    let mut result = Vec::<String>::new();
    result.push("".to_string());

    let mut is_split = true;

    for character in command.chars() {
        if character == '\'' || character == '\"' {
            is_split = !is_split;
            let index = result.len() - 1;
            result[index].push_str(&character.to_string());
        } else if character == ' ' && is_split {
            result.push("".to_string());
        } else {
            let index = result.len() - 1;
            result[index].push_str(&character.to_string());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_test() {
        let r = split("ls -la -h".to_string());
        assert_eq!(r.len(), 3);

        let r = split("ls -la --env=\"VAR VAR\" -h".to_string());
        assert_eq!(r.len(), 4);
    }
}
