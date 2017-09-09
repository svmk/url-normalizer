/*!
Purpose of this crate - URL normalization [WHATWG RFC] (https://tools.ietf.org/html/rfc3986#section-6)
*/
extern crate url;
use url::Url;
use std::collections::BinaryHeap;
use std::cmp::Ord;
use std::cmp::Ordering;

#[derive(PartialEq,Eq)]
struct Pair {
	key: String,
	value: String,
}

impl PartialOrd for Pair {
	fn partial_cmp(&self, other: &Pair) -> Option<Ordering> {
		return Some(cmp_string(&self.key, &other.key));
	}
}

impl Ord for Pair {
	fn cmp(&self, other: &Self) -> Ordering {
		return cmp_string(&self.key, &other.key);
	}
}

fn cmp_string(a:&String, b: &String) -> Ordering {
	let result = a.len().cmp(&b.len());
	match result {
		Ordering::Less => Ordering::Less,
		Ordering::Greater => Ordering::Greater,
		Ordering::Equal => {
			return a.cmp(b);
		}
	}
}

/// Normalizes URL
pub fn normalize(url: Url) -> Result<Url,()> {
	let url = normalize_query(url);
	let url = normalize_hash(url);
	return normalize_scheme(url);
}

/// Sorts url query in alphabet order.
pub fn normalize_query(mut url: Url) -> Url {
	let query_pairs: BinaryHeap<Pair> = url.query_pairs().into_owned().map(
		|(key,value)| {
			Pair {
				key: key,
				value: value,
			}
		}
	).collect();
	url.query_pairs_mut().clear();
	for pair in query_pairs.iter().rev() {
		url.query_pairs_mut().append_pair(&pair.key, &pair.value);
	}
	return url;
}

/// Removes hash part from url
pub fn normalize_hash(mut url: Url) -> Url {
	url.set_fragment(None);
	return url;
}

/// Changes encrypted scheme to unencrypted
pub fn normalize_scheme(mut url: Url) -> Result<Url,()> {
	let new_scheme;
	{
		let scheme = url.scheme();
		new_scheme = match scheme {
			"https" => Some("http"),
			"shttp" => Some("http"),
			"sftp" => Some("ftp"),
			"wss" => Some("ws"),
			_ => None,
		};
	}
	if let Some(scheme) = new_scheme {
		url.set_scheme(scheme)?;
	}
	return Ok(url);
}

#[cfg(test)]
mod tests {
	use super::*;
    #[test]
    fn test_normalize_query() {
        let url = Url::parse("https://example.com?c=1&q[]=99&q[5]=44&b=2&a=3#hash").unwrap();
        let url = normalize_query(url);
        assert_eq!(url.as_str(), "https://example.com/?a=3&b=2&c=1&q%5B%5D=99&q%5B5%5D=44#hash");
    }

    #[test]
    fn test_remove_hash() {
    	let url = Url::parse("https://example.com?c=1&q[]=99&q[5]=44&b=2&a=3#hash").unwrap();
    	let url = normalize_hash(url);
    	assert_eq!(url.as_str(),"https://example.com/?c=1&q[]=99&q[5]=44&b=2&a=3");
    }

    #[test]
    fn test_normalize_scheme() {
    	let url = Url::parse("https://example.com?c=1&q[]=99&q[5]=44&b=2&a=3#hash").unwrap();
    	let url = normalize_scheme(url).unwrap();
    	assert_eq!(url.as_str(),"http://example.com/?c=1&q[]=99&q[5]=44&b=2&a=3#hash");
    }

    fn process_normalize_scheme(secure_scheme: &str, scheme: &str) {
    	let url = Url::parse(&format!("{}://example.com/",secure_scheme)).unwrap();
    	let url = normalize_scheme(url).unwrap();
    	assert_eq!(url.as_str(), &format!("{}://example.com/", scheme));
    }

    #[test]
    fn test_normalize_scheme_https() {
    	process_normalize_scheme("https", "http");
    }

    #[test]
    fn test_normalize_scheme_wss() {
    	process_normalize_scheme("wss", "ws");
    }

    #[test]
    fn test_normalize_scheme_sftp() {
    	process_normalize_scheme("sftp", "ftp");
    }
}
