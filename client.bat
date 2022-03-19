wscat -c ws://84.30.14.3:25656

:: login User A {"type":"login","name":"User A"}
:: login User B {"type":"login","name":"User B"}
:: request      {"type":"request","target":""}
:: accept       {"type":"offer","accept":true,"id":""}
:: decline      {"type":"offer","accept":false,"id":""}