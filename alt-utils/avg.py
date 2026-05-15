import statistics
from prompt_toolkit import PromptSession  # type: ignore
from prompt_toolkit.history import InMemoryHistory  # type: ignore


def calc(s: str):
    try:
        data = [float(x.strip()) for x in s.split(',')]
    except ValueError:
        print("数据解析失败")
        return
    n = len(data)
    if n == 0:
        print("未检测到有效数据")
        return
    elif n == 1:
        avg = data[0]
        print(f"avg={avg:.2f} sd=N/A n=1")
        return
    avg = statistics.mean(data)
    sd = statistics.stdev(data)
    print(f"avg={avg:.2f} sd={sd:.2f} n={n}")


if __name__ == "__main__":
    session = PromptSession(history=InMemoryHistory())
    while True:
        try:
            user_input = session.prompt("calc> ")
            clean_input: str = user_input.strip().lower()
            if clean_input in ['q', 'exit', 'quit']:
                break
            if clean_input:
                for line in clean_input.splitlines(False):
                    calc(line)
        except KeyboardInterrupt:
            # ctrl c
            continue
        except EOFError:
            # ctrl d
            break
    pass
