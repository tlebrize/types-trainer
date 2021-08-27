import asyncio
import websockets

uri = "ws://localhost:8080"


async def pls():
    async with websockets.connect(uri) as ws:
        while True:
            p = input("?")
            if p != "wait":
                await ws.send(p)
            print(await ws.recv())


asyncio.get_event_loop().run_until_complete(pls())
