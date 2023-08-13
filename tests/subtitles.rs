/// Tests for subtitle support
///
/// We can run these tests on CI infrastructure because they are only downloading modest quantites
/// of data, corresponding to the subtitle files/MP4 fragments. This requires MP4Box (from GPAC) to
/// be installed on CI machines, however.

// To run tests while enabling printing to stdout/stderr, "cargo test -- --show-output" (from the
// root crate directory).


use std::fs;
use std::env;
use std::path::Path;
use dash_mpd::fetch::DashDownloader;

#[tokio::test]
async fn test_subtitles_wvtt () {
    let mpd = "https://storage.googleapis.com/shaka-demo-assets/sintel-mp4-wvtt/dash.mpd";
    let outpath = env::temp_dir().join("sintel.mp4");
    let mut subpath = outpath.clone();
    subpath.set_extension("srt");
    let subpath = Path::new(&subpath);
    assert!(DashDownloader::new(mpd)
            .fetch_audio(false)
            .fetch_video(false)
            .fetch_subtitles(true)
            .download_to(outpath.clone())
            .await
            .is_ok());
    assert!(fs::metadata(subpath).is_ok());
    let srt = fs::read_to_string(subpath).unwrap();
    // We didn't specify a preferred language, so the first available one in the manifest (here
    // Dutch) is downloaded.
    assert!(srt.contains("land van de poortwachters"));

    assert!(DashDownloader::new(mpd)
            .fetch_audio(false)
            .fetch_video(false)
            .fetch_subtitles(true)
            .prefer_language(String::from("eng"))
            .download_to(outpath.clone())
            .await
            .is_ok());
    let srt = fs::read_to_string(subpath).unwrap();
    // This time we requested English subtitles.
    assert!(srt.contains("land of the gatekeepers"));
}


#[tokio::test]
async fn test_subtitles_ttml () {
    let mpd = "https://dash.akamaized.net/dash264/TestCases/4b/qualcomm/2/TearsOfSteel_onDem5secSegSubTitles.mpd";
    let outpath = env::temp_dir().join("tears-of-steel.mp4");
    let mut subpath = outpath.clone();
    subpath.set_extension("ttml");
    let subpath = Path::new(&subpath);
    assert!(DashDownloader::new(mpd)
            .fetch_audio(false)
            .fetch_video(false)
            .fetch_subtitles(true)
            .download_to(outpath.clone())
            .await
            .is_ok());
    let ttml = fs::read_to_string(subpath).unwrap();
    // We didn't specify a preferred language, so the first available one in the manifest (here
    // English) is downloaded.
    assert!(ttml.contains("You're a jerk"));

    assert!(DashDownloader::new(mpd)
            .fetch_audio(false)
            .fetch_video(false)
            .fetch_subtitles(true)
            .prefer_language(String::from("de"))
            .download_to(outpath.clone())
            .await
            .is_ok());
    let ttml = fs::read_to_string(subpath).unwrap();
    // This time we requested German subtitles.
    assert!(ttml.contains("Du bist ein Vollidiot"));
}


// We can run this on CI infrastructure because it's only downloading a modest amount of subtitle
// segments.
#[tokio::test]
async fn test_subtitles_vtt () {
    let mpd = "http://dash.edgesuite.net/akamai/test/caption_test/ElephantsDream/elephants_dream_480p_heaac5_1.mpd";
    let outpath = env::temp_dir().join("elephants-dream.mp4");
    let mut subpath = outpath.clone();
    subpath.set_extension("vtt");
    let subpath = Path::new(&subpath);
    assert!(DashDownloader::new(mpd)
            .fetch_audio(false)
            .fetch_video(false)
            .fetch_subtitles(true)
            .prefer_language(String::from("de"))
            .download_to(outpath.clone())
            .await
            .is_ok());
    assert!(fs::metadata(subpath).is_ok());
    // This manifest contains a single subtitle track, available in VTT format via BaseURL addressing.
    let vtt = fs::read_to_string(subpath).unwrap();
    assert!(vtt.contains("Hurry Emo!"));
}


// STPP subtitles are muxed into the output media stream, so we need to download audio and video for
// this type. So we don't run this test on CI infrastructure.
#[tokio::test]
async fn test_subtitles_stpp() {
    use ffprobe::ffprobe;

    if env::var("CI").is_ok() {
        return;
    }
    let mpd = "https://rdmedia.bbc.co.uk/elephants_dream/1/client_manifest-all.mpd";
    let outpath = env::temp_dir().join("elephants-dream-bbc.mp4");
    assert!(DashDownloader::new(mpd)
            .fetch_audio(true)
            .fetch_video(true)
            .fetch_subtitles(true)
            .download_to(outpath.clone())
            .await
            .is_ok());
    let meta = ffprobe(outpath).unwrap();
    assert_eq!(meta.streams.len(), 3);
    let stpp = &meta.streams[2];
    assert_eq!(stpp.codec_tag_string, "stpp");
    let duration = stpp.duration.as_ref().unwrap().parse::<f64>().unwrap();
    assert!((620.0 < duration) && (duration < 640.0));
}



// https://media.axprod.net/TestVectors/Cmaf/clear_1080p_h264/manifest.mpd
// has vtt subs in de/en/fr