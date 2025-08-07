from yt_dlp import YoutubeDL
import json

def get_toplist(id: str):
    params = {'dump_single_json': True,
              'extract_flat': 'in_playlist',
              'noprogress': True,
              'quiet': True,
              'simulate': True}
    with YoutubeDL(params) as ydl:
        info = ydl.extract_info(f'https://music.163.com/#/discover/toplist?id={id}', download=False)
        return json.dumps(ydl.sanitize_info(info))



def get_song(url: str):
    params = {'dump_single_json': True,
              'extract_flat': False,
              'noprogress': True,
              'quiet': True,
              'simulate': True}
    with YoutubeDL(params) as ydl:
        info = ydl.extract_info(url, download=False)
        return json.dumps(ydl.sanitize_info(info))

