use std::sync::LazyLock;

use rand::Rng;

pub static STATUS_ONLINE: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "Terhubung",
        "Berselancar di Internet",
        "Online",
        "Aktif",
        "Masih Hidup",
        "Belum mati",
        "Belum ke-isekai",
        "Masih di Bumi",
        "Ada koneksi Internet",
        "Dar(l)ing",
        "Daring",
        "Bersama keluarga besar (Internet)",
        "Ngobrol",
        "Nge-meme bareng",
    ]
});
pub static STATUS_IDLE: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "Halo kau di sana?",
        "Ketiduran",
        "Nyawa di pertanyakan",
        "Halo????",
        "Riajuu mungkin",
        "Idle",
        "Gak aktif",
        "Jauh dari keyboard",
        "Lagi baper bentar",
        "Nonton Anime",
        "Lupa matiin data",
        "Lupa disconnect wifi",
        "Bengong",
    ]
});
pub static STATUS_DND: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "Lagi riajuu bentar",
        "Sibuk ~~onani/masturbasi~~",
        "Pacaran (joudan desu)",
        "Mungkin tidur",
        "Memantau keadaan",
        "Jadi satpam",
        "Mata-mata jadinya sibuk",
        "Bos besar supersibuk",
        "Ogah di-spam",
        "Nonton Anime",
        "Nonton Boku no Pico",
        "Nonton Dorama",
        "Sok sibuk",
        "Status kesukaan Kresbayyy",
        "Gangguin Mantan",
        "Ngestalk Seseorang",
        "Nge-roll gacha",
        "Nonton JAV",
        "Baca Doujinshi R-18++++",
        "Do not disturb",
        "Jangan ganggu",
        "Rapat DPR",
        "Sedang merencanakan UU baru",
        "Dangdutan bareng polisi",
    ]
});
pub static STATUS_OFFLINE: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "Mokad",
        "Off",
        "Tidak online",
        "Bosen hidup",
        "Dah bundir",
        "Dah di Isekai",
        "zzz",
        "Pura-pura off",
        "Invisible deng",
        "Memantau dari kejauhan",
        "Lagi comfy camping",
        "Riajuu selamanya",
        "Gak punya koneksi",
        "Gak ada sinyal",
        "Kuota habis",
    ]
});

fn select_random_from_vec(vec: &[&'static str]) -> String {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..vec.len());

    vec[index].to_string()
}

pub fn select_random_status(status: String) -> Option<String> {
    // valid status are:
    // online, idle, dnd, offline

    let status = status.to_lowercase();
    match status.as_str() {
        "online" => Some(select_random_from_vec(&STATUS_ONLINE)),
        "idle" => Some(select_random_from_vec(&STATUS_IDLE)),
        "dnd" => Some(select_random_from_vec(&STATUS_DND)),
        "offline" => Some(select_random_from_vec(&STATUS_OFFLINE)),
        _ => None,
    }
}
