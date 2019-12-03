# HTTP Requst Test

### Usage:

    $ request-test -c test.config


### Config Example:

    {
        "headers": {},
        "sync_list": [
            {
                "method": "get",
                "url": "http://127.0.0.1:8000/xxxx/",
                "data": {}
            },
            {
                "method": "post",
                "url": "http://127.0.0.1:8000/xxxx/",
                "data": {
                    "name": "test"
                }
            }
        ],
        "async_list": [
            {
                "method": "get",
                "url": "http://127.0.0.1:8000/xxxx/",
                "data": {}
            },
            {
                "method": "get",
                "url": "http://127.0.0.1:8000/xxxx/",
                "data": {}
            }
        ]
    }