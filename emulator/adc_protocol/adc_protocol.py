# protocol.py
# Реализация UDP-протокола виброоцифровщика
# Python 3.10
# Соответствует ТЗ п.5.3

import struct
from enum import IntEnum
from typing import List


# =========================
# Константы протокола
# =========================

class FunCode(IntEnum):
    SYN = 0x22
    DAT = 0x02
    CMD = 0x05
    ERR = 0x07


class Addr(IntEnum):
    SERVICE = 0x00
    ADC_DATA = 0x01
    CMD = 0x02


class DataType(IntEnum):
    ADC_12BIT = 0x10  


# =========================
# Вспомогательные функции
# =========================

def datatype_size(dtype: int) -> int:
    if dtype == 0x01:  # TYPE для управляющих команд
        return 1
    if dtype == DataType.ADC_12BIT:  # 0x10
        return 2
    raise ValueError(f"Неизвестный TYPE: {dtype:#02x}")


# =========================
# Формирование команд
# =========================

def build_start_command() -> bytes:
    """
    Команда START_TRANSFER
    FUN=0x05 ADDR=0x00 TYPE=0x01 COUNT=1 DATA=0x01
    """
    return struct.pack(
        ">BBBIB",
        FunCode.CMD,
        Addr.SERVICE,
        0x01,
        1,
        0x01
    )


def build_stop_command() -> bytes:
    """
    Команда STOP_TRANSFER
    FUN=0x05 ADDR=0x00 TYPE=0x01 COUNT=1 DATA=0x02
    """
    return struct.pack(
        ">BBBIB",
        FunCode.CMD,
        Addr.SERVICE,
        0x01,
        1,
        0x02
    )


# =========================
# Пакеты с данными АЦП
# =========================

def build_adc_packet(channel: int, samples: List[int]) -> bytes:
    """
    Формирование пакета с данными АЦП

    channel: 1 или 2
    samples: список значений АЦП (0..4095)
    """
    if channel not in (1, 2):
        raise ValueError("Канал должен быть 1 или 2")

    for s in samples:
        if not 0 <= s <= 0x0FFF:
            raise ValueError("Значение АЦП вне диапазона")

    header = struct.pack(
        ">BBB I",
        FunCode.DAT,
        channel,
        DataType.ADC_12BIT,
        len(samples)
    )

    data = struct.pack(f">{len(samples)}H", *samples)

    return header + data


# =========================
# Разбор входящих пакетов
# =========================

def parse_packet(packet: bytes, bytes_order=">BBB I") -> dict:
    """
    Разбор UDP-пакета по ТЗ
    Возвращает словарь с полями
    """
    if len(packet) < 7:
        raise ValueError("Пакет слишком короткий")

    fun, addr, dtype, count = struct.unpack(bytes_order, packet[:7])
    offset = 7

    #elem_size = datatype_size(dtype)
    expected_len = offset + count #* elem_size

    if len(packet) != expected_len:
        raise ValueError("Некорректная длина пакета")

    payload = packet[offset:]

    count = int(count/2)
    if dtype == DataType.ADC_12BIT:
        data = list(struct.unpack(f"<{count}H", payload))
    else:
        data = payload

    return {
        "fun": fun,
        "addr": addr,
        "type": dtype,
        "count": count,
        "data": data,
    }


# =========================
# Проверка команд
# =========================

def is_start_command(pkt: dict) -> bool:
    return (
        pkt["fun"] == FunCode.CMD and
        pkt["type"] == 0x01 and
        pkt.get("data") == b'\x01' 
    )

def is_stop_command(pkt: dict) -> bool:
    return (
        pkt["fun"] == FunCode.CMD and
        pkt["type"] == 0x01 and
        pkt.get("data") == b'\x02'
    )
