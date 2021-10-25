use git2::{Cred, Direction, Error, PushOptions, RemoteCallbacks, Repository, Signature};
use std::{
    fs::OpenOptions,
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

const SSH_KEY_LOCATION: &str = "../id_rsa";

fn configure_auth_callbacks(callbacks: &mut RemoteCallbacks) {
    // Prepare callbacks.
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.expect("Can't get username from URL"),
            None,
            std::path::Path::new(SSH_KEY_LOCATION),
            None,
        )
    });
}

fn clone_repo(repo_path: &Path, repo_url: &str) -> Result<Repository, Error> {
    std::fs::remove_dir_all(repo_path).expect("Can't remove target dir");

    let mut callbacks = RemoteCallbacks::new();
    configure_auth_callbacks(&mut callbacks);

    let mut fetch_opts = git2::FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fetch_opts);

    builder.clone(repo_url, repo_path)
}

fn modify_file_and_stage(repo_path: &Path, repo: &Repository) -> Result<(), Error> {
    let file_path = Path::new("README.md");

    let now = SystemTime::now();
    let time = now.duration_since(UNIX_EPOCH).expect("Can't make time");

    let full_path = repo_path.join(file_path);
    let mut file = OpenOptions::new()
        .append(true)
        .open(full_path)
        .expect("Unable to open file");

    let msg = format!("* Line added at {}\n", time.as_secs());
    file.write_all(msg.as_bytes()).expect("Can't write to file");

    let mut index = repo.index()?;
    index.add_path(file_path)?;

    Ok(())
}

fn commit(repo: &Repository) -> Result<(), Error> {
    let mut index = repo.index()?;
    let oid = index.write_tree()?;
    let signature = Signature::now("Robogit", "robogit@robogit.com")?;
    let parent_commit = repo.head().unwrap().peel_to_commit().unwrap();
    let tree = repo.find_tree(oid)?;
    let message = "Updated README.md";
    repo.commit(
        Some("HEAD"), // point HEAD to our new commit
        &signature,
        &signature,
        message,
        &tree,
        &[&parent_commit],
    )?;
    Ok(())
}

fn push(repo: &Repository) -> Result<(), Error> {
    let mut remote = repo
        .find_remote("origin")
        .expect("Can't find the 'origin' remote");

    let mut callbacks = RemoteCallbacks::new();
    configure_auth_callbacks(&mut callbacks);

    remote.connect_auth(Direction::Push, Some(callbacks), None)?;

    let mut callbacks = RemoteCallbacks::new();
    configure_auth_callbacks(&mut callbacks);

    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(callbacks);

    remote.push(
        &["refs/heads/master:refs/heads/master"],
        Some(&mut push_options),
    )
}

fn main() -> Result<(), Error> {
    let repo_path = Path::new("/tmp/robogit-patient");
    let repo_url = "git@github.com:zezic/robogit-patient.git";
    let repository = clone_repo(repo_path, repo_url)?;
    modify_file_and_stage(repo_path, &repository)?;
    commit(&repository)?;
    push(&repository)?;

    Ok(())
}
