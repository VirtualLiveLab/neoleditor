use crc32fast::Hasher;
use cobs;

#[tauri::command]
fn encode_packet(input: Vec<u8>) -> Vec<u8> {
    let mut hasher = Hasher::new();
    hasher.update(&input);
    let crc = hasher.finalize();

    let mut data_with_crc = input.clone();
    data_with_crc.extend_from_slice(&crc.to_be_bytes());

    let mut encoded = cobs::encode_vec(&data_with_crc);
    encoded.push(0x00);
    return encoded;
}

#[tauri::command]
fn decode_packet(input: Vec<u8>) -> Vec<u8> {
    let mut fallback = input.clone();
    if fallback.len() > 1 {
        fallback[1] = 0x02;
    }

    let data = if let Some((&0x00, rest)) = input.split_last() {
        rest
    } else {
        &input[..]
    };

    let decoded = match cobs::decode_vec(data) {
        Ok(d) => d,
        Err(_) => return fallback,
    };

    if decoded.len() < 4 {
        return fallback;
    }

    let (payload, crc_bytes) = decoded.split_at(decoded.len() - 4);
    let received_crc = u32::from_be_bytes(crc_bytes.try_into().unwrap());

    let mut hasher = crc32fast::Hasher::new();
    hasher.update(payload);
    let calculated_crc = hasher.finalize();

    if received_crc == calculated_crc {
        payload.to_vec()
    } else {
        let mut fallback2 = decoded.clone();
        if fallback2.len() > 1 {
            fallback2[1] = 0x03;
        }
        return fallback2;
    }
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_serialplugin::init())
        .invoke_handler(tauri::generate_handler![encode_packet, decode_packet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
