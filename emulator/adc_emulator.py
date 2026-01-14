#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Эмулятор устройства виброоцифровщика
Отправляет UDP пакеты с данными ADC для тестирования визуализации
"""

import socket
import struct
import time
import math
import argparse
import sys
from adc_protocol.adc_protocol import FunCode, DataType


class ADCEmulator:
    def __init__(self, target_ip="127.0.0.1", target_port=15180, sample_rate=1000):
        """
        Инициализация эмулятора
        
        Args:
            target_ip: IP адрес получателя (по умолчанию localhost)
            target_port: Порт получателя (по умолчанию 15180)
            sample_rate: Частота отправки пакетов (пакетов в секунду)
        """
        self.target_ip = target_ip
        self.target_port = target_port
        self.sample_rate = sample_rate
        self.sock = None
        self.running = False
        self.packet_counter = 0
        
        # Параметры генерации данных
        self.samples_per_packet = 256  # Сэмплов на канал в одном пакете
        self.adc_max_value = 4095  # Максимальное значение 12-битного ADC
        
    def _init_socket(self):
        """Инициализация UDP сокета"""
        try:
            self.sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            print(f"Сокет создан для отправки на {self.target_ip}:{self.target_port}")
        except Exception as e:
            print(f"Ошибка создания сокета: {e}")
            sys.exit(1)
    
    def generate_adc_data(self, packet_num):
        """
        Генерация тестовых данных ADC
        
        Args:
            packet_num: Номер пакета (для генерации различных паттернов)
            
        Returns:
            tuple: (adc0_samples, adc1_samples) - списки значений ADC
        """
        adc0_samples = []
        adc1_samples = []
        
        # Базовый индекс для генерации синусоидальных сигналов
        base_index = packet_num * self.samples_per_packet
        
        for i in range(self.samples_per_packet):
            # Глобальный индекс сэмпла
            sample_index = base_index + i
            
            # Генерация различных паттернов для тестирования
            # Частота дискретизации = sample_rate (пакетов/сек) * samples_per_packet (сэмплов/пакет)
            sampling_freq = self.sample_rate * self.samples_per_packet
            
            # ADC0: Синусоида с частотой 1 Гц
            freq0 = 10.0 / sampling_freq  # Нормализованная частота для 1 Гц
            phase0 = 2 * math.pi * freq0 * sample_index
            # Амплитуда 1500, смещение 2000
            adc0_val = int(2000 + 1500 * math.sin(phase0))
            adc0_val = max(0, min(self.adc_max_value, adc0_val))
            adc0_samples.append(adc0_val)
            
            # ADC1: Синусоида с частотой 0.5 Гц и сдвигом фазы
            freq1 = 200.0 / sampling_freq  # Нормализованная частота для 0.5 Гц
            phase1 = 2 * math.pi * freq1 * sample_index + math.pi / 4
            # Амплитуда 1000, смещение 2000
            adc1_val = int(2000 + 1000 * math.sin(phase1))
            adc1_val = max(0, min(self.adc_max_value, adc1_val))
            adc1_samples.append(adc1_val)
            
            # Добавляем небольшой шум для реалистичности
            import random
            noise0 = random.randint(-10, 10)
            noise1 = random.randint(-10, 10)
            adc0_samples[-1] = max(0, min(self.adc_max_value, adc0_samples[-1] + noise0))
            adc1_samples[-1] = max(0, min(self.adc_max_value, adc1_samples[-1] + noise1))
        
        return adc0_samples, adc1_samples
    
    def build_adc_packet(self, adc0_samples, adc1_samples):
        """
        Формирование пакета с данными ADC в формате, ожидаемом visual.py
        
        Формат пакета:
        - Заголовок: FUN (1 байт), ADDR (1 байт), TYPE (1 байт), COUNT (4 байта)
        - Данные: чередующиеся значения ADC0 и ADC1, по 2 байта каждое (little-endian)
        
        Args:
            adc0_samples: Список значений ADC0 (256 элементов)
            adc1_samples: Список значений ADC1 (256 элементов)
            
        Returns:
            bytes: Сформированный пакет
        """
        if len(adc0_samples) != self.samples_per_packet or len(adc1_samples) != self.samples_per_packet:
            raise ValueError(f"Количество сэмплов должно быть {self.samples_per_packet}")
        
        # Заголовок пакета (little-endian, как ожидает Rust client)
        # FUN=0x02 (DAT), channels=0x02 (2 channels), TYPE=0x10 (ADC_12BIT)
        # COUNT: количество байт данных = 512 значений * 2 байта = 1024 байта
        # После деления на 2 в parse_packet получится 512 значений
        header = struct.pack(
            "<BBB I",  # little-endian
            FunCode.DAT,
            0x02,  # channels: 2 channels (ADC0 and ADC1)
            DataType.ADC_12BIT,
            1024  # COUNT: 512 значений (256 ADC0 + 256 ADC1) * 2 байта = 1024 байта
        )
        
        # Данные: чередуются ADC0 и ADC1
        # [ADC0[0], ADC1[0], ADC0[1], ADC1[1], ...]
        data_bytes = bytearray()
        for i in range(self.samples_per_packet):
            # Убеждаемся, что значения в диапазоне 0-4095
            val0 = max(0, min(self.adc_max_value, adc0_samples[i])) & 0xFFF
            val1 = max(0, min(self.adc_max_value, adc1_samples[i])) & 0xFFF
            # Упаковка как 16-битные значения (little-endian)
            data_bytes.extend(struct.pack("<HH", val0, val1))
        
        return header + bytes(data_bytes)
    
    def send_packet(self, packet):
        """Отправка пакета через UDP"""
        try:
            self.sock.sendto(packet, (self.target_ip, self.target_port))
            if self.packet_counter == 0:
                # Выводим информацию о первом пакете для отладки
                print(f"Размер пакета: {len(packet)} байт")
                print(f"Заголовок (первые 7 байт): {packet[:7].hex()}")
                print(f"Первые 4 значения данных: {struct.unpack('<HHHH', packet[7:15])}")
            return True
        except Exception as e:
            print(f"Ошибка отправки пакета: {e}")
            return False
    
    def run(self, duration=None):
        """
        Запуск эмуляции
        
        Args:
            duration: Время работы в секундах (None = бесконечно)
        """
        self._init_socket()
        self.running = True
        
        packet_interval = 1.0 / self.sample_rate
        start_time = time.time()
        
        print(f"Эмулятор запущен")
        print(f"Отправка на {self.target_ip}:{self.target_port}")
        print(f"Частота: {self.sample_rate} пакетов/сек")
        print(f"Сэмплов на канал в пакете: {self.samples_per_packet}")
        print(f"Нажмите Ctrl+C для остановки")
        print("-" * 50)
        
        try:
            while self.running:
                # Проверка времени работы
                if duration is not None:
                    if time.time() - start_time >= duration:
                        break
                
                # Генерация данных
                adc0, adc1 = self.generate_adc_data(self.packet_counter)
                
                # Формирование пакета
                packet = self.build_adc_packet(adc0, adc1)
                
                # Отправка пакета
                if self.send_packet(packet):
                    self.packet_counter += 1
                    if self.packet_counter % 100 == 0:
                        print(f"Отправлено пакетов: {self.packet_counter}")
                
                # Задержка для поддержания частоты
                time.sleep(packet_interval)
                
        except KeyboardInterrupt:
            print("\nПолучен сигнал остановки...")
        finally:
            self.stop()
    
    def stop(self):
        """Остановка эмуляции"""
        self.running = False
        if self.sock:
            self.sock.close()
        print(f"\nЭмулятор остановлен. Всего отправлено пакетов: {self.packet_counter}")


def main():
    parser = argparse.ArgumentParser(
        description="Эмулятор устройства виброоцифровщика для тестирования визуализации"
    )
    parser.add_argument(
        "--ip",
        type=str,
        default="127.0.0.1",
        help="IP адрес получателя (по умолчанию: 127.0.0.1)"
    )
    parser.add_argument(
        "--port",
        type=int,
        default=15180,
        help="Порт получателя (по умолчанию: 15180)"
    )
    parser.add_argument(
        "--rate",
        type=float,
        default=1000.0,
        help="Частота отправки пакетов в секунду (по умолчанию: 1000)"
    )
    parser.add_argument(
        "--duration",
        type=float,
        default=None,
        help="Время работы в секундах (по умолчанию: бесконечно)"
    )
    
    args = parser.parse_args()
    
    emulator = ADCEmulator(
        target_ip=args.ip,
        target_port=args.port,
        sample_rate=args.rate
    )
    
    emulator.run(duration=args.duration)


if __name__ == "__main__":
    main()

