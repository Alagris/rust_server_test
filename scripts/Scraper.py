# from selenium import webdriver
# from selenium.webdriver.common.keys import Keys
from bs4 import BeautifulSoup
import requests
import pymongo

dbclient = pymongo.MongoClient("mongodb://localhost:27017/")

db = dbclient["boood"]

amazondb = db['products.amazon']

AMAZON_URL = "https://www.amazon.co.jp/"


# db.products.amazon.createIndex({"title":"text"})

#
# driver = webdriver.Firefox()

def get(url):
    return requests.get(url, headers={
        'user-agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_14_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.114 Safari/537.36',
        'sec-ch-ua': '"Google Chrome";v="89", "Chromium";v="89", ";Not A Brand";v="99"'
    })


def home_page():
    resp = get(AMAZON_URL)
    soup = BeautifulSoup(resp.content, 'html.parser')
    for link in soup.select("a[href]"):
        link = link["href"]
        if link.startswith("/") and not link.startswith("/gp"):
            print(link)
            amazondb.update_one({"url": link}, {'$set': {"url": link}}, upsert=True)


def product_page(url):
    resp = get(AMAZON_URL + url)
    soup = BeautifulSoup(resp.content, 'html.parser')
    title = soup.select("#productTitle")
    if title:
        title = title[0]
        mongo_obj = {'title': title.text.strip()}
        byline = soup.select("#bylineInfo_feature_div")
        if byline:
            byline = byline[0]
            byline_url = byline.select("a[href]")
            mongo_obj['byline'] = byline.text.strip()
            if byline_url:
                byline_url = [x['href'] for x in byline_url]
                mongo_obj['byline_url'] = byline_url
        price = soup.select("#priceblock_ourprice")
        if price:
            mongo_obj['price'] = price[0].text.strip()
        else:
            price = soup.select("#newBuyBoxPrice")
            if price:
                mongo_obj['price'] = price[0].text.strip()
        about = soup.select("#feature-bullets")
        if about:
            mongo_obj['about'] = about[0].text.strip()
        img = soup.select("#landingImage")
        if img:
            mongo_obj['img_url'] = img[0]['src']
        print("Updating ", mongo_obj)
        amazondb.update_one({"url": url}, {'$set': mongo_obj})
    else:
        print("Skipping")
        amazondb.update_one({"url": url}, {'$set': {'title': '?'}})


def all_missing():
    for missing in amazondb.find({'title': {'$exists': False}}):
        url = missing['url']
        print(url)
        product_page(url)


def set_up_indexing():
    amazondb.create_index({"title": "text"})


home_page()
all_missing()
# set_up_indexing()
