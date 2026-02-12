/// XMILE (XML Interchange Language for System Dynamics) parser
///
/// Supports XMILE v1.0 standard used by Stella, Vensim, and other SD tools

use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use crate::model::*;

pub fn parse_xmile(xml: &str) -> Result<Model, String> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut model = Model::new("Untitled Model");
    let mut stocks = Vec::new();
    let mut flows = Vec::new();
    let mut auxs = Vec::new();

    // Track current context
    let mut in_model = false;
    let mut in_variables = false;
    let mut current_stock: Option<XmileStock> = None;
    let mut current_flow: Option<XmileFlow> = None;
    let mut current_aux: Option<XmileAux> = None;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                match e.name().as_ref() {
                    b"xmile" => {
                        // Check version
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                if attr.key.as_ref() == b"version" {
                                    let version = String::from_utf8_lossy(&attr.value);
                                    if !version.starts_with("1.0") {
                                        eprintln!("Warning: XMILE version {} may not be fully supported", version);
                                    }
                                }
                            }
                        }
                    }
                    b"header" => {
                        // Will read name from inside
                    }
                    b"name" => {
                        if !in_model {
                            // Model name
                            if let Ok(Event::Text(e)) = reader.read_event_into(&mut buf) {
                                model.metadata.name = e.unescape().unwrap_or_default().to_string();
                            }
                        }
                    }
                    b"sim_specs" => {
                        // Parse simulation specs
                        parse_sim_specs(&mut reader, &mut model, &mut buf)?;
                    }
                    b"model" => {
                        in_model = true;
                    }
                    b"variables" => {
                        in_variables = true;
                    }
                    b"stock" => {
                        if in_variables {
                            let name = get_attribute(&e, b"name").unwrap_or_default();
                            current_stock = Some(XmileStock {
                                name,
                                eqn: String::new(),
                                inflows: Vec::new(),
                                outflows: Vec::new(),
                                units: None,
                                non_negative: false,
                                max_value: None,
                            });
                        }
                    }
                    b"non_negative" => {
                        // Set non_negative flag for current stock
                        if let Some(ref mut stock) = current_stock {
                            stock.non_negative = true;
                        }
                    }
                    b"max" => {
                        // Set max_value for current stock
                        if let Some(ref mut stock) = current_stock {
                            if let Ok(Event::Text(e)) = reader.read_event_into(&mut buf) {
                                if let Ok(max_val) = e.unescape().unwrap_or_default().parse::<f64>() {
                                    stock.max_value = Some(max_val);
                                }
                            }
                        }
                    }
                    b"flow" => {
                        if in_variables {
                            let name = get_attribute(&e, b"name").unwrap_or_default();
                            current_flow = Some(XmileFlow {
                                name,
                                eqn: String::new(),
                                units: None,
                            });
                        }
                    }
                    b"aux" => {
                        if in_variables {
                            let name = get_attribute(&e, b"name").unwrap_or_default();
                            current_aux = Some(XmileAux {
                                name,
                                eqn: String::new(),
                                units: None,
                            });
                        }
                    }
                    b"eqn" => {
                        // Read equation text
                        if let Ok(Event::Text(e)) = reader.read_event_into(&mut buf) {
                            let eqn = e.unescape().unwrap_or_default().to_string();

                            if let Some(ref mut stock) = current_stock {
                                stock.eqn = eqn;
                            } else if let Some(ref mut flow) = current_flow {
                                flow.eqn = eqn;
                            } else if let Some(ref mut aux) = current_aux {
                                aux.eqn = eqn;
                            }
                        }
                    }
                    b"inflow" => {
                        if let Some(ref mut stock) = current_stock {
                            if let Ok(Event::Text(e)) = reader.read_event_into(&mut buf) {
                                stock.inflows.push(e.unescape().unwrap_or_default().to_string());
                            }
                        }
                    }
                    b"outflow" => {
                        if let Some(ref mut stock) = current_stock {
                            if let Ok(Event::Text(e)) = reader.read_event_into(&mut buf) {
                                stock.outflows.push(e.unescape().unwrap_or_default().to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"stock" => {
                        if let Some(stock) = current_stock.take() {
                            stocks.push(stock);
                        }
                    }
                    b"flow" => {
                        if let Some(flow) = current_flow.take() {
                            flows.push(flow);
                        }
                    }
                    b"aux" => {
                        if let Some(aux) = current_aux.take() {
                            auxs.push(aux);
                        }
                    }
                    b"variables" => {
                        in_variables = false;
                    }
                    b"model" => {
                        in_model = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error at position {}: {}", reader.buffer_position(), e)),
            _ => {}
        }
        buf.clear();
    }

    // Convert XMILE structures to Model
    for xstock in stocks {
        let stock = Stock {
            name: xstock.name.clone(),
            initial: Expression::parse(&xstock.eqn)?,
            inflows: xstock.inflows,
            outflows: xstock.outflows,
            units: xstock.units,
            non_negative: xstock.non_negative,
            max_value: xstock.max_value,
            dimensions: None,
        };
        model.add_stock(stock)?;
    }

    for xflow in flows {
        let flow = Flow {
            name: xflow.name.clone(),
            equation: Expression::parse(&xflow.eqn)?,
            units: xflow.units,
        };
        model.add_flow(flow)?;
    }

    for xaux in auxs {
        let aux = Auxiliary {
            name: xaux.name.clone(),
            equation: Expression::parse(&xaux.eqn)?,
            units: xaux.units,
        };
        model.add_auxiliary(aux)?;
    }

    Ok(model)
}

fn parse_sim_specs(reader: &mut Reader<&[u8]>, model: &mut Model, buf: &mut Vec<u8>) -> Result<(), String> {
    let mut start = None;
    let mut stop = None;
    let mut dt = None;

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                match e.name().as_ref() {
                    b"start" => {
                        if let Ok(Event::Text(e)) = reader.read_event_into(buf) {
                            start = e.unescape().unwrap_or_default().parse().ok();
                        }
                    }
                    b"stop" => {
                        if let Ok(Event::Text(e)) = reader.read_event_into(buf) {
                            stop = e.unescape().unwrap_or_default().parse().ok();
                        }
                    }
                    b"dt" => {
                        if let Ok(Event::Text(e)) = reader.read_event_into(buf) {
                            dt = e.unescape().unwrap_or_default().parse().ok();
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"sim_specs" => break,
            Ok(Event::Eof) => return Err("Unexpected EOF in sim_specs".to_string()),
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    if let (Some(start_val), Some(stop_val), Some(dt_val)) = (start, stop, dt) {
        model.time.start = start_val;
        model.time.stop = stop_val;
        model.time.dt = dt_val;
    }

    Ok(())
}

fn get_attribute(element: &quick_xml::events::BytesStart, attr_name: &[u8]) -> Option<String> {
    element
        .attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.as_ref() == attr_name)
        .map(|a| String::from_utf8_lossy(&a.value).to_string())
}

// Intermediate structures for parsing
struct XmileStock {
    name: String,
    eqn: String,
    inflows: Vec<String>,
    outflows: Vec<String>,
    units: Option<String>,
    non_negative: bool,
    max_value: Option<f64>,
}

struct XmileFlow {
    name: String,
    eqn: String,
    units: Option<String>,
}

struct XmileAux {
    name: String,
    eqn: String,
    units: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_xmile() {
        let xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <xmile version="1.0">
            <header>
                <name>Simple Model</name>
            </header>
            <sim_specs>
                <start>0</start>
                <stop>10</stop>
                <dt>1</dt>
            </sim_specs>
            <model>
                <variables>
                    <stock name="Population">
                        <eqn>100</eqn>
                        <inflow>births</inflow>
                    </stock>
                    <flow name="births">
                        <eqn>Population * 0.1</eqn>
                    </flow>
                </variables>
            </model>
        </xmile>
        "#;

        let model = parse_xmile(xml).unwrap();
        assert_eq!(model.metadata.name, "Simple Model");
        assert_eq!(model.time.start, 0.0);
        assert_eq!(model.time.stop, 10.0);
        assert_eq!(model.time.dt, 1.0);
        assert_eq!(model.stocks.len(), 1);
        assert_eq!(model.flows.len(), 1);
    }
}
