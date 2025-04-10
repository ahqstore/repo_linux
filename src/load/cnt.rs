use content_disposition;
use reqwest::header::CONTENT_DISPOSITION;

pub fn guess_filename(resp: &reqwest::Response) -> Option<String> {
  let dsp = content_disposition::parse_content_disposition(
    resp.headers().get(CONTENT_DISPOSITION)?.to_str().ok()?,
  );

  let filename = dsp.filename_full();

  drop(dsp);

  filename
}
