use clap;
use lopdf::{
    // content::{Content, Operation},
    dictionary,
};

const A4_W: f64 = 595.0;
const A4_H: f64 = 842.0;

// The CliArgs struct holds the parsed and validated command-line arguments.
// This provides a clean interface for the main application logic.
struct CliArgs {
    input_file: String,
    output_file: String,
    selections: Vec<String>,
}

#[repr(usize)]
#[derive(Debug)]
enum Side {Left, Bottom, Right, Top, N, }

#[derive(Clone, Default, Debug)]
struct Selection {
    page_number : u32,
    margin_width: [u32; Side::N as usize],
}
impl std::fmt::Display for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.page_number)?;
        for i in 0..4 {
            write!(f, ":{}", self.margin_width[i])?;
        }
        Ok(()) 
    }
}

impl Selection {
    fn new_or_default(s: &str, default_selection: &Selection) -> Result<Self, String> {
        let mut err_msg : String = String::new();
        let ss : Vec<String> = s.split(':').map(|s| s.to_string()).collect();
        let sslen = ss.len();
        if sslen < 1 || 5 < sslen {
            err_msg = format!("Number of colon-separated values in {} is {}, must be within [1,5]",
                s, sslen);
        }
        let mut ret : Selection = default_selection.clone();
        if err_msg.is_empty() {
            match ss[0].parse::<u32>() {
                Ok(num) => { ret.page_number = num },
                Err(e) => { err_msg = e.to_string() },
            }
        }
        let mut i : usize = 0;
        while err_msg.is_empty() && i + 1 < sslen {
            if !ss[i + 1].is_empty() {
                match ss[i + 1].parse::<u32>() {
                    Ok(num) => { ret.margin_width[i] = num },
                    Err(e) => { err_msg = e.to_string() },
                }
            }
            i += 1;
        }
        if err_msg.is_empty() {
            Ok(ret)
        } else {
            Err(err_msg)
        }
    }
}

fn is_valid_selection(val: &str) -> Result<String, String> {
    let default_selection = Selection{ page_number: 0, margin_width: [0, 0, 0, 0], };
    match Selection::new_or_default(val, &default_selection) {
        Ok(_) => Ok(val.to_string()),
        Err(err_msg) => Err(err_msg),
    }
}

// This function is dedicated to parsing the command-line arguments.
// It returns a `Result` to allow the caller to handle parsing failures.
fn parse_arguments() -> Result<CliArgs, clap::Error> {
    let matches = clap::Command::new("pdf select and clean margins")
        .version("1.0")
        .author("You")
        .about("A simple application that processes files.")
        .arg(
            clap::Arg::new("input")
                .short('i')
                .long("input")
                .value_name("input.pdf")
                .help("pdf input")
                .required(true),
        )
        .arg(
            clap::Arg::new("output")
                .short('o')
                .long("output")
                .value_name("output.pdf")
                .help("pdf output")
                .required(true),
        )
        .arg(
            clap::Arg::new("selection")
                .short('s')
                .long("selection")
                .value_name("PageSpec")
                .help("(repeateable) pagenum:left:bottom:right:top")
                .action(clap::ArgAction::Append)
                .required(true)
                .value_parser(is_valid_selection), // This now uses our new parsing function
        )
        .try_get_matches()?; // Use `try_get_matches` to return a Result

    let input_file = matches.get_one::<String>("input").unwrap().to_string();
    let output_file = matches.get_one::<String>("output").unwrap().to_string();

    let selections: Vec<String> = matches.get_many::<String>("selection")
                                        .unwrap()
                                        .map(|s| s.to_string())
                                        .collect();

    Ok(CliArgs {
        input_file,
        output_file,
        selections,
    })
}

fn clone_object(
    src: &lopdf::Document,
    dst: &mut lopdf::Document,
    id: lopdf::ObjectId,
    map: &mut std::collections::HashMap<lopdf::ObjectId, lopdf::ObjectId>,
) -> anyhow::Result<lopdf::ObjectId> {
    if let Some(&new_id) = map.get(&id) {
        return Ok(new_id);
    }

    let new_id = dst.new_object_id();
    map.insert(id, new_id);

    let obj = src.get_object(id)?.clone();

    let new_obj = match obj {
        lopdf::Object::Reference(r) => lopdf::Object::Reference(clone_object(src, dst, r, map)?),

        lopdf::Object::Array(arr) => {
            lopdf::Object::Array(arr.into_iter().map(|o| clone_obj_rec(src, dst, o, map)).collect::<anyhow::Result<_>>()?)
        }

        lopdf::Object::Dictionary(mut dict) => {
            for (_, v) in dict.iter_mut() {
                *v = clone_obj_rec(src, dst, v.clone(), map)?;
            }
            lopdf::Object::Dictionary(dict)
        }

        lopdf::Object::Stream(mut s) => {
            for (_, v) in s.dict.iter_mut() {
                *v = clone_obj_rec(src, dst, v.clone(), map)?;
            }
            lopdf::Object::Stream(s)
        }

        other => other,
    };

    dst.objects.insert(new_id, new_obj);
    Ok(new_id)
}

fn clone_obj_rec(
    src: &lopdf::Document,
    dst: &mut lopdf::Document,
    obj: lopdf::Object,
    map: &mut std::collections::HashMap<lopdf::ObjectId, lopdf::ObjectId>,
) -> anyhow::Result<lopdf::Object> {
    Ok(match obj {
        lopdf::Object::Reference(r) => lopdf::Object::Reference(clone_object(src, dst, r, map)?),
        lopdf::Object::Array(arr) => {
            lopdf::Object::Array(arr.into_iter().map(|o| clone_obj_rec(src, dst, o, map)).collect::<anyhow::Result<_>>()?)
        }
        lopdf::Object::Dictionary(mut dict) => {
            for (_, v) in dict.iter_mut() {
                *v = clone_obj_rec(src, dst, v.clone(), map)?;
            }
            lopdf::Object::Dictionary(dict)
        }
        other => other,
    })
}

fn import_page_as_xobject(
    out: &mut lopdf::Document,
    src: &mut lopdf::Document,
    page_num: u32,
) -> anyhow::Result<lopdf::ObjectId> {
    let mut map = std::collections::HashMap::new();

    let pages = src.get_pages();
    let page_id = pages.get(&page_num).ok_or_else(|| anyhow::anyhow!("page not found"))?;

    let content_data = src.get_page_content(*page_id)?;

    let (res_opt, _) = src.get_page_resources(*page_id)?;

    let resources = if let Some(res_dict) = res_opt {
        match clone_obj_rec(src, out, lopdf::Object::Dictionary(res_dict.clone()), &mut map)? {
            lopdf::Object::Dictionary(d) => d,
            _ => lopdf::Dictionary::new(),
        }
    } else {
        lopdf::Dictionary::new()
    };

    let form = lopdf::Stream::new(
        lopdf::dictionary! {
            "Type" => "XObject",
            "Subtype" => "Form",
            "BBox" => vec![0.into(), 0.into(), A4_W.into(), A4_H.into()],
            "Resources" => resources,
        },
        content_data,
    );

    Ok(out.add_object(form))
}

fn build_output_page(
    out: &mut lopdf::Document,
    form_id: lopdf::ObjectId,
    overlay: bool,
) -> anyhow::Result<lopdf::ObjectId> {
    let mut ops = vec![
        lopdf::content::Operation::new("q", vec![]),
        lopdf::content::Operation::new("Do", vec![lopdf::Object::Name(b"Fm0".to_vec())]),
        lopdf::content::Operation::new("Q", vec![]),
    ];

    if overlay {
        let w = A4_W / 2.0;
        let h = A4_H / 2.0;
        let x = (A4_W - w) / 2.0;
        let y = (A4_H - h) / 2.0;

        ops.extend(vec![
            lopdf::content::Operation::new("q", vec![]),
            lopdf::content::Operation::new("1 0 0 rg", vec![]),
            lopdf::content::Operation::new("re", vec![x.into(), y.into(), w.into(), h.into()]),
            lopdf::content::Operation::new("f", vec![]),
            lopdf::content::Operation::new("Q", vec![]),
        ]);
    }

    let content = lopdf::content::Content { operations: ops };
    let content_id = out.add_object(lopdf::Stream::new(lopdf::dictionary! {}, content.encode()?));

    let resources_id = out.add_object(lopdf::dictionary! {
        "XObject" => lopdf::dictionary! {
            "Fm0" => lopdf::Object::Reference(form_id)
        }
    });

    let page_id = out.new_object_id();
    out.objects.insert(
        page_id,
        lopdf::Object::Dictionary(dictionary! {
            "Type" => "Page",
            "MediaBox" => vec![0.into(), 0.into(), A4_W.into(), A4_H.into()],
            "Contents" => lopdf::Object::Reference(content_id),
            "Resources" => lopdf::Object::Reference(resources_id),
        }),
    );

    Ok(page_id)
}

fn select_and_clean(args: &CliArgs) {
    println!("Processing input file: {}", args.input_file);
    println!("Writing to output file: {}", args.output_file);

    match lopdf::Document::load(args.input_file.clone()) {
       Ok(doc) => {
            let mut doc_out = lopdf::Document::with_version("1.5");
            let pages_id_out = doc_out.new_object_id();
            let pages = lopdf::dictionary! {};

            let page_ids: Vec<lopdf::ObjectId> = doc.get_pages().into_values().collect();
            // println!("page_ids: {:?}", page_ids);
            println!("{} pages in {}", page_ids.len(), args.input_file);

            println!("Selected items:");
            let mut selection_prev = Selection{ page_number: 0, margin_width: [0, 0, 0, 0], };
            for s_selection in &args.selections {
                let mut selection = Selection::new_or_default(s_selection, &selection_prev)
                    .unwrap();
                println!("selection={} page_number={}", selection, selection.page_number);
                selection_prev = selection;
            }

            doc_out.objects.insert(pages_id_out, lopdf::Object::Dictionary(pages));
            doc_out.save(args.output_file.clone()).unwrap();
       },
       Err(msg) => { println!("Failed to load {}, {}", args.input_file, msg) },
    }

}

fn main() {
    match parse_arguments() {
        Ok(args) => {
            select_and_clean(&args);
        }
        Err(e) => {
            e.exit();
        }
    }
}
