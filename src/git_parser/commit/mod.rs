use nom::{
    bytes::complete::{tag, take_till, take_until},
    character::complete::line_ending,
    combinator::{map, opt, rest},
    sequence::{preceded, terminated},
    IResult,
};
use std::str;

#[derive(Debug)]
pub struct GitCommitEntry {
    tree: String,
    parent: Option<String>,
    author: GitUser,
    committer: GitUser,
    gpgsig: Option<String>,
    message: String,
}

#[derive(Debug)]
pub struct GitUser {
    name: String,
    email: String,
    timestamp: String,
}

fn parse_single_line(input: &[u8]) -> IResult<&[u8], &str> {
    map(take_till(|c| c == 0x0A || c == 0x0D), |d: &[u8]| {
        str::from_utf8(d).expect("")
    })(input)
}

fn parse_tree(input: &[u8]) -> IResult<&[u8], &str> {
    preceded(tag("tree "), parse_single_line)(input)
}

fn parse_parent(input: &[u8]) -> IResult<&[u8], &str> {
    preceded(tag("parent "), parse_single_line)(input)
}

fn parse_user(input: &[u8]) -> IResult<&[u8], GitUser> {
    let (input, name) = take_till(|c| c == b'<')(input)?;
    let (input, email) = preceded(tag("<"), take_till(|c| c == b'>'))(input)?;
    let (input, _) = tag("> ")(input)?;
    let (input, timestamp) = take_till(|c| c == b'\n')(input)?;
    Ok((
        input,
        GitUser {
            name: str::from_utf8(name).expect("").trim().to_string(),
            email: str::from_utf8(email).expect("").to_string(),
            timestamp: str::from_utf8(timestamp).expect("").to_string(),
        },
    ))
}

fn parse_author(input: &[u8]) -> IResult<&[u8], GitUser> {
    preceded(tag("author "), parse_user)(input)
}

fn parse_committer(input: &[u8]) -> IResult<&[u8], GitUser> {
    preceded(tag("committer "), parse_user)(input)
}

fn parse_gpgsig(input: &[u8]) -> IResult<&[u8], &[u8]> {
    terminated(
        preceded(
            tag("gpgsig -----BEGIN PGP SIGNATURE-----"),
            take_until("-----END PGP SIGNATURE-----"),
        ),
        tag("-----END PGP SIGNATURE-----"),
    )(input)
}

// 9785695
pub fn parse_git_commit_entry(input: &[u8]) -> IResult<&[u8], GitCommitEntry> {
    let (input, tree) = terminated(parse_tree, line_ending)(input)?;
    let (input, parent) = opt(terminated(parse_parent, line_ending))(input)?;
    let (input, _parent_2) = opt(terminated(parse_parent, line_ending))(input)?;
    let (input, _parent_3) = opt(terminated(parse_parent, line_ending))(input)?;
    let (input, author) = terminated(parse_author, line_ending)(input)?;
    let (input, committer) = terminated(parse_committer, line_ending)(input)?;
    let (input, gpgsig) = opt(terminated(parse_gpgsig, line_ending))(input)?;
    let (input, message) = rest(input)?;

    Ok((
        input,
        GitCommitEntry {
            tree: tree.to_string(),
            parent: parent.map(|p| p.to_string()),
            author,
            committer,
            gpgsig: gpgsig.map(|g| str::from_utf8(g).expect("").to_string()),
            message: str::from_utf8(message).expect("").to_string(),
        },
    ))
}
