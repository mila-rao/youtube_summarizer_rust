# YouTube Video Summariser

This tool summarises any YouTube video (under 45 minutes playtime only). You simply feed the video url and in a couple of minutes (depending on the length of the video and the availability of the API) you have a summarised text of the video. The tool makes use of YouTube Transcript API library (python native) for video transcripts and the facebook/bart-large-cnn model for summarisation. Note that this tool works only for YouTube videos. 

For more information on YouTube Transcript API:
https://github.com/jdepoix/youtube-transcript-api

For more information on facebook/bart-large-cnn model:
https://huggingface.co/facebook/bart-large-cnn

## Running the Tool Locally
If you wish to run the tool locally, you need Rust (1.79.0 minimum) and python3. 

### Installing rust
The best way to install rust is via rustup - https://rustup.rs/
This downloads the compiler, cargo and rustdoc. The website contains step by step instructions. 

### Installing python
The best way is to download python from the download centre on the python website - https://www.python.org/downloads/

Alternatively, if you are a Mac user, you can do brew install
```
$(brew --prefix python)/libexec/bin
```

Make sure to also install the setup tools 
```
python3 -m pip install --upgrade setuptools
```

### Cloning the repo locally

To run this tool locally, you need to clone this repository. Input the below command in your terminal after travelling to the desired directory (where you want to store this repo). 
```
https://github.com/mila-rao/youtube_summarizer_rust.git
```

### Installing dependencies
Once the repo is cloned, you need to install the python dependencies. Don't worry about the rust dependencies as cargo will take care of everything. 

Either run the requirements.txt file
```
pip install -r requirements.txt
```
Or, manually install the YouTube Transcript API
```
pip install youtube-transcript-api==0.6.1
```

### Getting HuggingFace access token
The tool does not run the facebook/bart-large-cnn model locally but calls the API endpoint for it. The endpoint is provided via HuggingFace, which is a model repository. To call it, an access token is needed. For this, follow the steps below:

1. Go to https://huggingface.co/ and open an account
2. Once the account is setup, click on the account profile picture and go to Settings
3. Go to Access Tokens
4. Before creating a new one, create a new json file in the youtube_summarizer folder of the cloned repo. The json file should contain your access token in the following key-value pair.
```
{
    "token": "hf_your_token_here"
}
```
5. Go back to HuggingFace and create an access token. You need a Read token and can give it any name. Copy the token and paste it in the json file in place of hf_your_token_here. It must be inside the double quote marks ""

### Running the tool
Once all the previous steps are complete, the tool can be run locally. Open the repo folder in an IDE of your choice. VD Code is recommended by any IDE that supports rust is fine. 

In the teriminal of the IDE run the following command
```
cargo build
```

This will install all the dependencies and compile the code. Once this is done sucessfully and without errors, run the below command
```
cargo run
```
Enter the url of the video when prompted. 
The summary should be available in few minutes

## Troubleshooting common errors
```
Error: Failed to send request to API: https://api-inference.huggingface.co/models/facebook/bart-large-cnn: status code 503
```
If the API is unresponsive, try again in a few minutes. 