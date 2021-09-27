use std::env;
use std::error;
use std::fs;
use toml::Value;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

macro_rules! unwrap_or_return {
    ( $e:expr, $v: expr ) => {
        match $e {
            Some(x) => x,
            None => return $v,
        }
    };
}

fn benchmark_to_csv_rows(benchmark: &Value) -> Vec<String> {
    let results = unwrap_or_return!(
        benchmark
            .get("results")
            .and_then(|results| results.as_array())
            .and_then(|results| results.get(0)) // There's just one block
            .and_then(|results| results.as_table()),
        vec![]
    );
    let app = unwrap_or_return!(benchmark.get("app").and_then(|app| app.as_table()), vec![]);
    let options = unwrap_or_return!(app.get("options").and_then(|o| o.as_array()), vec![]);
    let tags = options
        .iter()
        .find_map(|option| {
            option.get("name").and_then(|name| {
                if name.as_str() == Some("tags") {
                    option.get("contents")
                } else {
                    None
                }
            })
        })
        .and_then(|c| c.as_array())
        .and_then(|tags| tags.get(0))
        .and_then(|tags| tags.as_str())
        .and_then(|tag_string| {
            let mut split_iter = tag_string.split('-');
            let left: i64 = unwrap_or_return!(split_iter.next().and_then(|l| l.parse().ok()), None);
            let right: i64 =
                unwrap_or_return!(split_iter.next().and_then(|l| l.parse().ok()), None);
            Some((left, right))
        });

    let (left_tag, right_tag) = tags.unwrap();

    let ms_run: Vec<f64> = results
        .get("ms_run")
        .and_then(|x| x.as_array())
        .unwrap()
        .iter()
        .flat_map(|xs| xs.as_float())
        .collect();

    let parameters: Vec<&str> = results
        .get("post")
        .and_then(|post| post.as_array())
        .and_then(|x| x.get(0))
        .and_then(|t| t.get("output"))
        .and_then(|o| o.get(0))
        .and_then(|output| output.as_str())
        .map(|output| {
            let mut out_iter = output.split('\n');
            let nr_values: usize = out_iter
                .next()
                .and_then(|nr_parameters| nr_parameters.parse().ok())
                .unwrap();
            out_iter.take(nr_values).collect()
        })
        .unwrap();
    ms_run
        .iter()
        .map(|runtime| {
            format!(
                "{},{},{},{}",
                runtime,
                left_tag,
                right_tag,
                parameters.join("/")
            )
        })
        .collect()
}

fn toml_to_csv(results: Value) -> Result<Vec<String>> {
    results
        .as_table()
        .and_then(|root_table| root_table.get("benchmark"))
        .and_then(|bs| bs.as_array())
        .map(|bs| {
            bs.iter()
                .flat_map(benchmark_to_csv_rows)
                .collect::<Vec<String>>()
        })
        .ok_or_else(|| "Invalid format".into())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let program_name = &args[0];
    let filename = args
        .get(1)
        .ok_or(format!("Usage: {} <path>", program_name))?;
    let contents = fs::read_to_string(filename)?;
    let results: Value = contents.parse()?;
    println!("runtime,left_tag,right_tag,parameters");
    for line in toml_to_csv(results)? {
        println!("{}", line);
    }
    Ok(())
}
