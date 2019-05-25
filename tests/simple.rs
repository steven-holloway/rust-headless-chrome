#![allow(unused_variables)]

use std::sync::Arc;

use base64;
use log::*;
use rand::prelude::*;

use headless_chrome::browser::tab::RequestInterceptionDecision;
use headless_chrome::protocol::network::methods::RequestPattern;
use headless_chrome::{
    browser::default_executable, browser::tab::Tab, protocol::page::ScreenshotFormat, Browser,
    LaunchOptionsBuilder,
    protocol::page::EmulateMediaOptions
};
use std::thread::sleep;
use std::time::Duration;

mod logging;
mod server;

/// Launches a dumb server that unconditionally serves the given data as a
/// successful html response; launches a new browser and navigates to the
/// server.
///
/// Users must hold on to the server, which stops when dropped.
fn dumb_server(data: &'static str) -> (server::Server, Browser, Arc<Tab>) {
    let server = server::Server::with_dumb_html(data);
    let (browser, tab) = dumb_client(&server);
    (server, browser, tab)
}

fn dumb_client(server: &server::Server) -> (Browser, Arc<Tab>) {
    let browser = Browser::new(
        LaunchOptionsBuilder::default()
            .path(Some(default_executable().unwrap()))
            .build()
            .unwrap(),
    )
        .unwrap();
    let tab = browser.wait_for_initial_tab().unwrap();
    tab.navigate_to(&format!("http://127.0.0.1:{}", server.port()))
        .unwrap();
    (browser, tab)
}

#[test]
fn simple() -> Result<(), failure::Error> {
    logging::enable_logging();
    let (server, browser, tab) = dumb_server(include_str!("simple.html"));
    tab.wait_for_element("div#foobar")?;
    Ok(())
}

#[test]
fn actions_on_tab_wont_hang_after_browser_drops() -> Result<(), failure::Error> {
    logging::enable_logging();
    for _ in 0..20 {
        let (_, browser, tab) = dumb_server(include_str!("simple.html"));
        std::thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let millis: u64 = rng.gen_range(0, 5000);
            std::thread::sleep(std::time::Duration::from_millis(millis));
            trace!("dropping browser");
            drop(browser);
        });
        let _element = tab.find_element("div#foobar");
    }
    Ok(())
}

#[test]
fn form_interaction() -> Result<(), failure::Error> {
    logging::enable_logging();
    let (_, browser, tab) = dumb_server(include_str!("form.html"));
    tab.wait_for_element("input#target")?
        .type_into("mothership")?;
    tab.wait_for_element("button")?.click()?;
    let d = tab.wait_for_element("div#protocol")?.get_description()?;
    assert!(d
        .find(|n| n.node_value == "Missiles launched against mothership")
        .is_some());
    tab.wait_for_element("input#sneakattack")?.click()?;
    tab.wait_for_element("button")?.click()?;
    let d = tab.wait_for_element("div#protocol")?.get_description()?;
    assert!(d
        .find(|n| n.node_value == "Comrades, have a nice day!")
        .is_some());
    Ok(())
}

fn decode_png(i: &[u8]) -> Result<Vec<u8>, failure::Error> {
    let decoder = png::Decoder::new(&i[..]);
    let (info, mut reader) = decoder.read_info()?;
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf)?;
    Ok(buf)
}

fn sum_of_errors(inp: &[u8], fixture: &[u8]) -> u32 {
    inp.chunks_exact(fixture.len())
        .map(|c| {
            c.iter()
                .zip(fixture)
                .map(|(b, e)| (i32::from(*b) - i32::from(*e)).pow(2) as u32)
                .sum::<u32>()
        })
        .sum()
}

#[test]
fn capture_screenshot_png() -> Result<(), failure::Error> {
    logging::enable_logging();
    let (_, browser, tab) = dumb_server(include_str!("simple.html"));
    tab.wait_for_element("div#foobar")?;
    // Check that the top-left pixel on the page has the background color set in simple.html
    let png_data = tab.capture_screenshot(ScreenshotFormat::PNG, None, true)?;
    let buf = decode_png(&png_data[..])?;
    assert!(sum_of_errors(&buf[0..4], &[0x11, 0x22, 0x33, 0xff]) < 5);
    Ok(())
}

#[test]
fn capture_screenshot_element() -> Result<(), failure::Error> {
    logging::enable_logging();
    let (_, browser, tab) = dumb_server(include_str!("simple.html"));
    // Check that the screenshot of the div's content-box has no other color than the one set in simple.html
    let png_data = tab
        .wait_for_element("div#foobar")?
        .capture_screenshot(ScreenshotFormat::PNG)?;
    let buf = decode_png(&png_data[..])?;
    for i in 0..buf.len() / 4 {
        assert!(sum_of_errors(&buf[i * 4..(i + 1) * 4], &[0x33, 0x22, 0x11, 0xff]) < 5);
    }
    Ok(())
}

#[test]
fn capture_screenshot_element_box() -> Result<(), failure::Error> {
    logging::enable_logging();
    let (_, browser, tab) = dumb_server(include_str!("simple.html"));
    // Check that the top-left pixel of the div's border-box has the border's color set in simple.html
    let pox = tab.wait_for_element("div#foobar")?.get_box_model()?;
    let png_data =
        tab.capture_screenshot(ScreenshotFormat::PNG, Some(pox.border_viewport()), true)?;
    let buf = decode_png(&png_data[..])?;
    assert!(dbg!(sum_of_errors(&buf[0..4], &[0x22, 0x11, 0x33, 0xff])) < 15);
    Ok(())
}

#[test]
fn capture_screenshot_jpeg() -> Result<(), failure::Error> {
    logging::enable_logging();
    let (_, browser, tab) = dumb_server(include_str!("simple.html"));
    tab.wait_for_element("div#foobar")?;
    let jpg_data = tab.capture_screenshot(ScreenshotFormat::JPEG(Some(100)), None, true)?;
    let mut decoder = jpeg_decoder::Decoder::new(&jpg_data[..]);
    let buf = decoder.decode().unwrap();
    assert!(sum_of_errors(&buf[0..4], &[0x11, 0x22, 0x33]) < 5);
    Ok(())
}

#[test]
fn test_print_file_to_pdf() -> Result<(), failure::Error> {
    logging::enable_logging();
    let (_, browser, tab) = dumb_server(include_str!("./pdfassets/index.html"));
    let local_pdf = tab.wait_until_navigated()?.print_to_pdf(None)?;
    assert_eq!(true, local_pdf.len() > 1000); // an arbitrary size
    assert!(local_pdf.starts_with(b"%PDF"));
    Ok(())
}

#[test]
fn test_emulate_media() -> Result<(), failure::Error> {
    logging::enable_logging();
    let options = EmulateMediaOptions {
        media_type: "screen".to_string()
    };

    let (_, browser, tab) = dumb_server(include_str!("./pdfassets/index.html"));
    let response = tab.wait_until_navigated()?.emulate_media(Some(options))?;

//    async emulateMedia(mediaType) {
//        assert(mediaType === 'screen' || mediaType === 'print' || mediaType === null, 'Unsupported media type: ' + mediaType);
//        await this._client.send('Emulation.setEmulatedMedia', {media: mediaType || ''});
//    }


    Ok(())
}

#[test]
fn get_box_model() -> Result<(), failure::Error> {
    logging::enable_logging();
    let (_, browser, tab) = dumb_server(include_str!("simple.html"));
    let pox = tab.wait_for_element("div#foobar")?.get_box_model()?;
    // Check that the div has exactly the dimensions we set in simple.html
    assert_eq!(pox.width, 3 + 100 + 3);
    assert_eq!(pox.height, 3 + 20 + 3);
    Ok(())
}

#[test]
fn box_model_geometry() -> Result<(), failure::Error> {
    logging::enable_logging();
    let (_, browser, tab) = dumb_server(include_str!("simple.html"));
    let center = tab.wait_for_element("div#position-test")?.get_box_model()?;
    let within = tab.wait_for_element("div#within")?.get_box_model()?;
    let above = tab
        .wait_for_element("div#strictly-above")?
        .get_box_model()?;
    let below = tab
        .wait_for_element("div#strictly-below")?
        .get_box_model()?;
    let left = tab.wait_for_element("div#strictly-left")?.get_box_model()?;
    let right = tab
        .wait_for_element("div#strictly-right")?
        .get_box_model()?;

    assert!(above.content.strictly_above(&center.content));
    assert!(above.content.above(&center.content));
    assert!(above.margin.above(&center.content));
    assert!(!above.margin.strictly_above(&center.content));
    assert!(above.content.within_horizontal_bounds_of(&center.content));
    assert!(!above.content.within_vertical_bounds_of(&center.content));

    assert!(below.content.strictly_below(&center.content));
    assert!(below.content.below(&center.content));
    assert!(below.margin.below(&center.content));
    assert!(!below.margin.strictly_below(&center.content));

    assert!(left.content.strictly_left_of(&center.content));
    assert!(left.content.left_of(&center.content));
    assert!(left.margin.left_of(&center.content));
    assert!(!left.margin.strictly_left_of(&center.content));
    assert!(!left.content.within_horizontal_bounds_of(&center.content));
    assert!(left.content.within_vertical_bounds_of(&center.content));

    assert!(right.content.strictly_right_of(&center.content));
    assert!(right.content.right_of(&center.content));
    assert!(right.margin.right_of(&center.content));
    assert!(!right.margin.strictly_right_of(&center.content));

    assert!(within.content.within_bounds_of(&center.content));
    assert!(!center.content.within_bounds_of(&within.content));

    Ok(())
}

#[test]
fn reload() -> Result<(), failure::Error> {
    logging::enable_logging();
    let mut counter = 0;
    let responder = move |r: tiny_http::Request| {
        let response = tiny_http::Response::new(
            200.into(),
            vec![tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap()],
            std::io::Cursor::new(format!(r#"<div id="counter">{}</div>"#, counter)),
            None,
            None,
        );
        trace!("{}", counter);
        counter += 1;
        r.respond(response)
    };
    let server = server::Server::new(responder);
    let (browser, tab) = dumb_client(&server);
    assert!(tab
        .wait_for_element("div#counter")?
        .get_description()?
        .find(|n| n.node_value == "0")
        .is_some());
    assert!(tab
        .reload(false, None)?
        .wait_for_element("div#counter")?
        .get_description()?
        .find(|n| n.node_value == "1")
        .is_some());
    // TODO test effect of scriptEvaluateOnLoad
    Ok(())
}

#[test]
fn find_elements() -> Result<(), failure::Error> {
    logging::enable_logging();
    let (server, browser, tab) = dumb_server(include_str!("simple.html"));
    let divs = tab.wait_for_elements("div")?;
    assert_eq!(8, divs.len());
    Ok(())
}

#[test]
fn call_js_fn_sync() -> Result<(), failure::Error> {
    logging::enable_logging();
    let (server, browser, tab) = dumb_server(include_str!("simple.html"));
    let element = tab.wait_for_element("#foobar")?;
    let result = element.call_js_fn("function() { return 42 }", false)?;
    assert_eq!(result.object_type, "number");
    assert_eq!(result.description, Some("42".to_owned()));
    assert_eq!(result.value, Some((42).into()));
    Ok(())
}

#[test]
fn call_js_fn_async_unresolved() -> Result<(), failure::Error> {
    logging::enable_logging();
    let (server, browser, tab) = dumb_server(include_str!("simple.html"));
    let element = tab.wait_for_element("#foobar")?;
    let result = element.call_js_fn("async function() { return 42 }", false)?;
    assert_eq!(result.object_type, "object");
    assert_eq!(result.subtype, Some("promise".to_owned()));
    assert_eq!(result.description, Some("Promise".to_owned()));
    assert_eq!(result.value, None);
    Ok(())
}

#[test]
fn call_js_fn_async_resolved() -> Result<(), failure::Error> {
    logging::enable_logging();
    let (server, browser, tab) = dumb_server(include_str!("simple.html"));
    let element = tab.wait_for_element("#foobar")?;
    let result = element.call_js_fn("async function() { return 42 }", true)?;
    assert_eq!(result.object_type, "number");
    assert_eq!(result.subtype, None);
    assert_eq!(result.description, Some("42".to_owned()));
    assert_eq!(result.value, Some((42).into()));
    Ok(())
}

#[test]
fn set_request_interception() -> Result<(), failure::Error> {
    logging::enable_logging();
    let server = server::Server::with_dumb_html(include_str!(
        "coverage_fixtures/basic_page_with_js_scripts.html"
    ));

    let browser = Browser::new(
        LaunchOptionsBuilder::default()
            .path(Some(default_executable().unwrap()))
            .build()
            .unwrap(),
    )
        .unwrap();

    let tab = browser.wait_for_initial_tab().unwrap();

    //    tab.call_method(network::methods::Enable{})?;

    let patterns = vec![
        RequestPattern {
            url_pattern: None,
            resource_type: None,
            interception_stage: Some("HeadersReceived"),
        },
        RequestPattern {
            url_pattern: None,
            resource_type: None,
            interception_stage: Some("Request"),
        },
    ];

    tab.enable_request_interception(
        &patterns,
        Box::new(|transport, session_id, intercepted| {
            if intercepted.request.url.ends_with(".js") {
                let js_body = r#"document.body.appendChild(document.createElement("hr"));"#;
                let js_response = tiny_http::Response::new(
                    200.into(),
                    vec![tiny_http::Header::from_bytes(
                        &b"Content-Type"[..],
                        &b"application/javascript"[..],
                    )
                        .unwrap()],
                    js_body.as_bytes(),
                    Some(js_body.len()),
                    None,
                );

                let mut wrapped_writer = Vec::new();
                js_response
                    .raw_print(&mut wrapped_writer, (1, 2).into(), &[], false, None)
                    .unwrap();

                let base64_response = base64::encode(&wrapped_writer);

                RequestInterceptionDecision::Response(base64_response)
            } else {
                RequestInterceptionDecision::Continue
            }
        }),
    )?;

    // ignore cache:
    tab.navigate_to(&format!("http://127.0.0.1:{}", server.port()))
        .unwrap();

    tab.wait_until_navigated()?;

    // There are two JS scripts that get loaded via network, they both append an element like this:
    assert_eq!(2, tab.wait_for_elements("hr")?.len());

    Ok(())
}

#[test]
fn incognito_contexts() -> Result<(), failure::Error> {
    logging::enable_logging();
    let (server, browser, tab) = dumb_server(include_str!("simple.html"));

    let incognito_context = browser.new_context()?;
    let incognito_tab: Arc<Tab> = incognito_context.new_tab()?;
    let tab_context_id = incognito_tab.get_target_info()?.browser_context_id.unwrap();

    assert_eq!(incognito_context.get_id(), tab_context_id);
    assert_eq!(
        incognito_context.get_tabs()?[0].get_target_id(),
        incognito_tab.get_target_id()
    );
    Ok(())
}

#[test]
fn get_script_source() -> Result<(), failure::Error> {
    logging::enable_logging();
    let server = server::file_server("tests/coverage_fixtures");
    let browser = Browser::new(
        LaunchOptionsBuilder::default()
            .path(Some(default_executable().unwrap()))
            .headless(false)
            .build()
            .unwrap(),
    )
        .unwrap();

    let tab: Arc<Tab> = browser.wait_for_initial_tab()?;

    tab.enable_profiler()?;
    tab.start_js_coverage()?;

    tab.navigate_to(&format!(
        "{}/basic_page_with_js_scripts.html",
        &server.url()
    ))?;

    tab.wait_until_navigated()?;

    sleep(Duration::from_millis(100));

    let script_coverages = tab.take_precise_js_coverage()?;

    tab.enable_debugger()?;

    let contents = tab.get_script_source(&script_coverages[0].script_id)?;
    assert_eq!(
        include_str!("coverage_fixtures/coverage_fixture1.js"),
        contents
    );

    let contents = tab.get_script_source(&script_coverages[1].script_id)?;
    assert_eq!(
        include_str!("coverage_fixtures/coverage_fixture2.js"),
        contents
    );

    Ok(())
}
