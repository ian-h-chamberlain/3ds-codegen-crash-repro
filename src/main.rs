use regex::RegexBuilder;

const RE: &str = r"(?P<key>.+)=(?P<value>.+)";

fn main() {
    pthread_3ds::init();
    linker_fix_3ds::init();

    let builder = RegexBuilder::new(RE);
    let _regex = builder.build();
}
