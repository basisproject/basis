"use strict";

const rp = require('request-promise');
const Exonum = require('exonum-client');
const config = require('../helpers/config');
const protobuf = require('../helpers/protobuf');
const standard_api = require('../helpers/standard-api');

exports.generate = function(path, type) {
	function verify(res) {
		const Type = Exonum.newType(protobuf.root.lookupType(type));
		try {
			const obj_proof = new Exonum.MapProof(res.item_proof.object, Exonum.Hash, Type);
			const tbl_proof = new Exonum.MapProof(res.item_proof.table, Exonum.Hash, Exonum.Hash);
			if(res.item) {
				const root_hash = Exonum.uint8ArrayToHexadecimal(new Uint8Array(res.item.history_hash.data));
				const len = res.item.history_len;
				const tree_proof = Exonum.merkleProof(root_hash, len, res.item_history.proof, [0, len], Type);
			}
		} catch(e) {
			console.log('err proof: ', e);
		}
	}

	return {
		list: async function(qs) {
			let res = await rp({
				url: `${config.endpoint}/services/factor/v1${path}`,
				json: true,
				qs: qs,
			});
			return (res && res.items) || [];
		},

		get: async function(qs, options) {
			options || (options = {});
			const {extended} = options;
			let res = await rp({
				url: `${config.endpoint}/services/factor/v1${path}/info`,
				json: true,
				qs: qs,
			});
			if(!res) return null;
			verify(res);
			if(!extended) res = res.item;
			return res;
		},
	};
};


