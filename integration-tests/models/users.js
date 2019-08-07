"use strict";

const rp = require('request-promise');
const Exonum = require('exonum-client');
const config = require('../helpers/config');
const protobuf = require('../helpers/protobuf');

function verify(res) {
	const User = Exonum.newType(protobuf.root.lookupType('factor.user.User'));
	try {
		const obj_proof = new Exonum.MapProof(res.item_proof.object, Exonum.Hash, User);
		const tbl_proof = new Exonum.MapProof(res.item_proof.table, Exonum.Hash, Exonum.Hash);
		if(res.item) {
			const root_hash = Exonum.uint8ArrayToHexadecimal(new Uint8Array(res.item.history_hash.data));
			const len = res.item.history_len;
			const tree_proof = Exonum.merkleProof(root_hash, len, res.item_history.proof, [0, len], User);
		}
	} catch(e) {
		console.log('err proof: ', e);
	}
}

exports.list = async function({after, per_page}) {
	let res = await rp({
		url: `${config.endpoint}/services/factor/v1/users`,
		json: true,
		qs: {
			after: after,
			per_page: per_page,
		},
	});
	return (res && res.items) || [];
};

exports.get = async function({id, pubkey, email}, options) {
	options || (options = {});
	const {extended} = options;
	let res = await rp({
		url: `${config.endpoint}/services/factor/v1/users/info`,
		json: true,
		qs: {
			id: id,
			pubkey: pubkey,
			email: email,
		},
	});
	if(!res) return null;
	verify(res);
	if(!extended) res = res.item;
	return res;
};

