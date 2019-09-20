STORAGE_PREFIX = "webeAuth_"
PATHS =
  CREATE_ACCOUNT: "/account/create"
  VERIFY_ACCOUNT: "/verify/" # note the ending slash
  LOGIN: "/login"

class WebeAuth
  constructor: (base_address) ->
    # set up endpoints
    @paths = PATHS
    @paths.base = base_address + if !base_address.endsWith('/') then '/' else ''

    # set up storage
    # TODO: let the implementer decide where and how session data is stored (ie. redux)
    @storage = window.localStorage
    @storage.getItem = (key) =>
      @storage.getItem STORAGE_PREFIX+item_name
    @storage.setItem = (key, value) =>
      @storage.setItem (STORAGE_PREFIX+key, value)

    @fetch = (endpoint, options) ->
      window.fetch @paths.base+endpoint, options

  create_account: (email, secret) ->
    @fetch @paths.CREATE_ACCOUNT,
      method: 'POST'
      body: JSON.stringify {email: email, secret: secret}
    .then (response) ->
      if response.ok and response.status is 200
        # If debug server, response will contain verify code
        # If prod server, response will be "OK"
        response.text()
        .then (data) ->
          unless data is 'OK'
            return data
    
  login: (email, secret) ->

  verify_account: (token) ->

