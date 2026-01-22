import * as React from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { cn } from "@/lib/utils";

export type CalendarProps = {
  selected?: Date;
  onSelect?: (date: Date) => void;
  className?: string;
};

export function Calendar({ selected, onSelect, className }: CalendarProps) {
  const [currentMonth, setCurrentMonth] = React.useState(
    selected || new Date()
  );

  const daysInMonth = new Date(
    currentMonth.getFullYear(),
    currentMonth.getMonth() + 1,
    0
  ).getDate();

  const firstDayOfMonth = new Date(
    currentMonth.getFullYear(),
    currentMonth.getMonth(),
    1
  ).getDay();

  const monthNames = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
  ];

  const dayNames = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

  const previousMonth = () => {
    setCurrentMonth(
      new Date(currentMonth.getFullYear(), currentMonth.getMonth() - 1, 1)
    );
  };

  const nextMonth = () => {
    setCurrentMonth(
      new Date(currentMonth.getFullYear(), currentMonth.getMonth() + 1, 1)
    );
  };

  const handleDateClick = (day: number) => {
    const date = new Date(
      currentMonth.getFullYear(),
      currentMonth.getMonth(),
      day
    );
    onSelect?.(date);
  };

  const isToday = (day: number) => {
    const today = new Date();
    return (
      day === today.getDate() &&
      currentMonth.getMonth() === today.getMonth() &&
      currentMonth.getFullYear() === today.getFullYear()
    );
  };

  const isSelected = (day: number) => {
    if (!selected) return false;
    return (
      day === selected.getDate() &&
      currentMonth.getMonth() === selected.getMonth() &&
      currentMonth.getFullYear() === selected.getFullYear()
    );
  };

  const renderDays = () => {
    const days = [];
    for (let i = 0; i < firstDayOfMonth; i++) {
      days.push(<div key={`empty-${i}`} className="h-9 w-9" />);
    }
    for (let day = 1; day <= daysInMonth; day++) {
      days.push(
        <button
          key={day}
          onClick={() => handleDateClick(day)}
          className={cn(
            "h-9 w-9 rounded-md text-sm font-medium transition-colors",
            "hover:bg-accent hover:text-accent-foreground",
            "focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
            isToday(day) && "bg-primary text-primary-foreground font-bold",
            isSelected(day) &&
              !isToday(day) &&
              "bg-accent text-accent-foreground ring-2 ring-primary"
          )}
        >
          {day}
        </button>
      );
    }
    return days;
  };

  return (
    <div className={cn("p-3", className)}>
      <div className="flex items-center justify-between mb-4">
        <button
          onClick={previousMonth}
          className="h-7 w-7 rounded-md hover:bg-accent hover:text-accent-foreground"
        >
          <ChevronLeft className="h-4 w-4" />
        </button>
        <div className="text-sm font-semibold">
          {monthNames[currentMonth.getMonth()]} {currentMonth.getFullYear()}
        </div>
        <button
          onClick={nextMonth}
          className="h-7 w-7 rounded-md hover:bg-accent hover:text-accent-foreground"
        >
          <ChevronRight className="h-4 w-4" />
        </button>
      </div>
      <div className="grid grid-cols-7 gap-1 mb-2">
        {dayNames.map((name) => (
          <div
            key={name}
            className="h-9 w-9 text-xs font-medium text-muted-foreground flex items-center justify-center"
          >
            {name}
          </div>
        ))}
      </div>
      <div className="grid grid-cols-7 gap-1">{renderDays()}</div>
    </div>
  );
}
