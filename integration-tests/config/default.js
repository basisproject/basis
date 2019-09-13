module.exports = {
	// where to find our api
	endpoint: 'http://127.0.0.1:13007/api',

	// where our proto buffers are located (only change this if you have a good
	// reason to)
	protobuf_dir: `${__dirname}/../../bundle/models/src/proto`,

	// the intial user we create to facilitate the creation of all our test
	// conditions and data
	bootstrap_user: {
		// arbitrary, but should probs be a uuid
		id: false,
		// arbitrary, but should be a real email
		email: false,
		// set this to the same value as ../config/config.yaml::tests.bootstrap_user_key
		// to generate a new keypair, use `node tools/keygen.js`
		pub: false,
		// set to the secret key paired to the bootstrap_user.pub key
		sec: false,
	},

	// should stay 128 likely
	service_id: 128,
};

