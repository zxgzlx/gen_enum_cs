use askama::Template;
use calamine::{open_workbook, Reader, Xlsx};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write};

#[derive(Deserialize)]
struct Code {
    #[serde(rename = "##var")]
    var: Option<String>,
    #[serde(rename = "type")]
    type_name: Option<String>,
    remark: Option<String>,
}

#[derive(Template)]
#[template(path = "code.txt")]
struct CodeTemplate<'a> {
    namespace: &'a str,
    classname: &'a str,
    tuples: &'a Vec<(&'a str, &'a str, &'a str)>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub headers: Vec<String>,
    pub work_sheet_name: String,
    pub class_name: String,
    pub in_path: String,
    pub out_path: String,
    pub namespace: String,
}

pub fn gen_code() {
    let inputs = load_input_from_json().unwrap();
    if inputs.len() == 0 {
        return;
    }
    println!("{:#?}", inputs);
    for input in inputs {
        handle_input(input).unwrap();
    }
}

fn load_input_from_json() -> Result<Vec<Input>, Box<dyn std::error::Error>> {
    // 读取文件内容
    let json_data = fs::read_to_string("inputs.json")?;

    // 将 JSON 数据反序列化为 Input 结构体
    let inputs: Vec<Input> = serde_json::from_str(&json_data)?;

    Ok(inputs)
}

fn handle_input(input: Input) -> Result<(), Box<dyn std::error::Error>> {
    let in_path = input.in_path;

    let mut excel: Xlsx<_> = open_workbook(in_path)?;

    let range = excel
        .worksheet_range(input.work_sheet_name.as_str())
        .map_err(|_| calamine::Error::Msg("Cannot find Sheet1"))?;

    let mut header_cells: Vec<usize> = vec![];
    for row in range.headers() {
        // println!("{:?}", row);
        header_cells = row
            .iter()
            .enumerate()
            .filter(|(_, x)| input.headers.contains(x))
            .map(|(i, _)| i)
            .collect();
        break;
    }

    let mut code_tuples = vec![];
    for row in range.rows() {
        let mut collect_row = vec![];
        for header_index in header_cells.iter() {
            if *header_index == 0 {
                if row[0].to_string().starts_with("##") {
                    break;
                }
                collect_row.push("string".to_string());
                continue;
            }

            let remark = row[*header_index]
                .to_string()
                .replace("\n", "\n        /// ");
            collect_row.push(remark);
        }
        if collect_row.len() == 0 {
            continue;
        }
        // println!("{:?}", collect_row);
        code_tuples.push(collect_row);
    }

    let tuples_vec = code_tuples
        .iter()
        .map(|x| (x[0].as_str(), x[1].as_str(), x[2].as_str()))
        .collect();
    // println!("{:?}", tuples_vec);
    let code_template = CodeTemplate {
        namespace: input.namespace.as_str(),
        classname: input.class_name.as_str(),
        tuples: &tuples_vec,
    };

    // 保存json文件
    let out_path = input.out_path.as_str();
    if std::path::Path::new(out_path).exists() {
        std::fs::remove_file(out_path).unwrap();
    }

    let mut file = std::fs::File::create(out_path).unwrap();
    file.write_all(code_template.render().unwrap().as_bytes())?;
    println!("生成代码成功：{}", out_path);

    Ok(())
}

mod test {
    use super::*;

    #[test]
    fn test_gen_code() {
        gen_code();
    }
}
