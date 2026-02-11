import React from "react";
import { Calendar } from "lucide-react";

const CalendarPage = () => {
  return (
    <div className="h-full flex items-center justify-center">
      <div className="flex flex-col items-center gap-4 text-center max-w-md">
        <div className="h-12 w-12 rounded-full bg-muted flex items-center justify-center">
          <Calendar className="h-6 w-6 text-muted-foreground" />
        </div>
        <h1 className="text-2xl font-bold">Calendar View</h1>
        <p className="text-muted-foreground">
          View your documents and notes organized by date. See your activity
          timeline and navigate through your content by calendar. This feature
          is coming soon.
        </p>
      </div>
    </div>
  );
};

export default CalendarPage;
