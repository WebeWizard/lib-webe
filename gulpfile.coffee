{src, dest, pipe, series} = require 'gulp'
rename = require 'gulp-rename'
{spawn} = require 'child_process'
process = require 'process'

serverProcess = null

devEnv = ->
  src('dev.env').pipe(rename '.env').pipe dest '.'

runExample = (cb) ->
  serverProcess = spawn 'cargo', ['run','-p','example'], {stdio: 'inherit'}
  setTimeout -> # give the server some time to start up
    cb()
  , 5000

killServer = (cb) ->
  process.kill(serverProcess.pid) if serverProcess?
  cb()

module.exports =
  
  testExample: series devEnv, runExample, killServer

  

  