// Import hacspec and all needed definitions.
use hacspec_lib::*;

// Import aes and gcm
// use super::{self, aes_ctr_keyblock, aes_encrypt, Block};
// use super::gf128::{gmac, Key, Tag};
use hacspec_aes::*;
use hacspec_gf128::*;

fn pad_aad_msg(aad: &ByteSeq, msg: &ByteSeq) -> ByteSeq {
    let laad = aad.len();
    let lmsg = msg.len();
    let pad_aad = if laad % 16 == 0 {
        laad
    } else {
        laad + (16 - (laad % 16))
    };
    let pad_msg = if lmsg % 16 == 0 {
        lmsg
    } else {
        lmsg + (16 - (lmsg % 16))
    };
    let mut padded_msg = ByteSeq::new(pad_aad + pad_msg + 16);
    padded_msg = padded_msg.update(0, aad);
    padded_msg = padded_msg.update(pad_aad, msg);
    padded_msg = padded_msg.update(
        pad_aad + pad_msg,
        &U64_to_be_bytes(U64(laad as u64) * U64(8u64)),
    );
    padded_msg = padded_msg.update(
        pad_aad + pad_msg + 8,
        &U64_to_be_bytes(U64(lmsg as u64) * U64(8u64)),
    );
    padded_msg
}

pub(crate) fn encrypt_aes(
    key: &ByteSeq,
    iv: Nonce,
    aad: &ByteSeq,
    msg: &ByteSeq,
) -> (ByteSeq, Tag) {
    let iv0 = Nonce::new();

    let (_success, mac_key) = aes_ctr_keyblock(
        key,
        iv0,
        U32(0u32),
        KEY_LENGTH,
        ROUNDS,
        KEY_SCHEDULE_LENGTH,
        KEY_LENGTH,
        ITERATIONS,
    );
    let (_success, tag_mix) = aes_ctr_keyblock(
        key,
        iv.clone(), // FIXME: is not necessary.
        U32(1u32),
        KEY_LENGTH,
        ROUNDS,
        KEY_SCHEDULE_LENGTH,
        KEY_LENGTH,
        ITERATIONS,
    );

    let cipher_text = aes128_encrypt(Key128::from_seq(key), iv, U32(2u32), msg);
    let padded_msg = pad_aad_msg(aad, &cipher_text);
    let tag = gmac(&padded_msg, Gf128Key::from_seq(&mac_key));
    let tag = xor_block(Block::from_seq(&tag), tag_mix);

    (cipher_text, Tag::from_seq(&tag))
}

pub fn encrypt_aes128(key: Key128, iv: Nonce, aad: &ByteSeq, msg: &ByteSeq) -> (ByteSeq, Tag) {
    encrypt_aes(&ByteSeq::from_seq(&key), iv, aad, msg)
}

pub(crate) fn decrypt_aes(
    key: &ByteSeq,
    iv: Nonce,
    aad: &ByteSeq,
    cipher_text: &ByteSeq,
    tag: Tag,
) -> (bool, ByteSeq) {
    let iv0 = Nonce::new();

    let (_success, mac_key) = aes_ctr_keyblock(
        key,
        iv0,
        U32(0u32),
        KEY_LENGTH,
        ROUNDS,
        KEY_SCHEDULE_LENGTH,
        KEY_LENGTH,
        ITERATIONS,
    );
    let (_success, tag_mix) = aes_ctr_keyblock(
        key,
        iv.clone(), // FIXME: is not necessary.
        U32(1u32),
        KEY_LENGTH,
        ROUNDS,
        KEY_SCHEDULE_LENGTH,
        KEY_LENGTH,
        ITERATIONS,
    );

    let padded_msg = pad_aad_msg(aad, cipher_text);
    let my_tag = gmac(&padded_msg, Gf128Key::from_seq(&mac_key));
    let my_tag = xor_block(Block::from_seq(&my_tag), tag_mix);

    if my_tag.declassify_eq(&Block::from_seq(&tag)) {
        (
            true,
            aes128_decrypt(Key128::from_seq(key), iv, U32(2u32), cipher_text),
        )
    } else {
        (false, ByteSeq::new(0))
    }
}

pub fn decrypt_aes128(
    key: Key128,
    iv: Nonce,
    aad: &ByteSeq,
    cipher_text: &ByteSeq,
    tag: Tag,
) -> (bool, ByteSeq) {
    decrypt_aes(&ByteSeq::from_seq(&key), iv, aad, cipher_text, tag)
}
