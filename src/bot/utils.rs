pub fn parse_msg_for_tg(message: String) -> String {
    message
        .replace('-', "\\-")
        // .replace('_', "\\_")
        // .replace('*', "\\*")
        // .replace('`', "\\`")
        .replace('.', "\\.")
}
