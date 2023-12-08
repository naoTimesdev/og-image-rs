use std::vec;

use lazy_static::lazy_static;
use rand::Rng;

lazy_static! {
    pub static ref STATUS_ONLINE: Vec<String> = vec![
        "Terhubung".to_string(),
        "Berselancar di Internet".to_string(),
        "Online".to_string(),
        "Aktif".to_string(),
        "Masih Hidup".to_string(),
        "Belum mati".to_string(),
        "Belum ke-isekai".to_string(),
        "Masih di Bumi".to_string(),
        "Ada koneksi Internet".to_string(),
        "Dar(l)ing".to_string(),
        "Daring".to_string(),
        "Bersama keluarga besar (Internet)".to_string(),
        "Ngobrol".to_string(),
        "Nge-meme bareng".to_string(),
    ];
    pub static ref STATUS_IDLE: Vec<String> = vec![
        "Halo kau di sana?".to_string(),
        "Ketiduran".to_string(),
        "Nyawa di pertanyakan".to_string(),
        "Halo????".to_string(),
        "Riajuu mungkin".to_string(),
        "Idle".to_string(),
        "Gak aktif".to_string(),
        "Jauh dari keyboard".to_string(),
        "Lagi baper bentar".to_string(),
        "Nonton Anime".to_string(),
        "Lupa matiin data".to_string(),
        "Lupa disconnect wifi".to_string(),
        "Bengong".to_string(),
    ];
    pub static ref STATUS_DND: Vec<String> = vec![
        "Lagi riajuu bentar".to_string(),
        "Sibuk ~~onani/masturbasi~~".to_string(),
        "Pacaran (joudan desu)".to_string(),
        "Mungkin tidur".to_string(),
        "Memantau keadaan".to_string(),
        "Jadi satpam".to_string(),
        "Mata-mata jadinya sibuk".to_string(),
        "Bos besar supersibuk".to_string(),
        "Ogah di-spam".to_string(),
        "Nonton Anime".to_string(),
        "Nonton Boku no Pico".to_string(),
        "Nonton Dorama".to_string(),
        "Sok sibuk".to_string(),
        "Status kesukaan Kresbayyy".to_string(),
        "Gangguin Mantan".to_string(),
        "Ngestalk Seseorang".to_string(),
        "Nge-roll gacha".to_string(),
        "Nonton JAV".to_string(),
        "Baca Doujinshi R-18++++".to_string(),
        "Do not disturb".to_string(),
        "Jangan ganggu".to_string(),
        "Rapat DPR".to_string(),
        "Sedang merencanakan UU baru".to_string(),
        "Dangdutan bareng polisi".to_string(),
    ];
    pub static ref STATUS_OFFLINE: Vec<String> = vec![
        "Mokad".to_string(),
        "Off".to_string(),
        "Tidak online".to_string(),
        "Bosen hidup".to_string(),
        "Dah bundir".to_string(),
        "Dah di Isekai".to_string(),
        "zzz".to_string(),
        "Pura-pura off".to_string(),
        "Invisible deng".to_string(),
        "Memantau dari kejauhan".to_string(),
        "Lagi comfy camping".to_string(),
        "Riajuu selamanya".to_string(),
        "Gak punya koneksi".to_string(),
        "Gak ada sinyal".to_string(),
        "Kuota habis".to_string(),
    ];
}

fn select_random_from_vec(vec: &Vec<String>) -> String {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..vec.len());
    let result = vec[index].clone();
    result
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
